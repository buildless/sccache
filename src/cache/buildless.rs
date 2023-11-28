// Copyright 2016 Mozilla Foundation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::Path;

use opendal::layers::LoggingLayer;
use opendal::services::Webdav;
use opendal::Operator;

use cache::redis::RedisCache;

use serde::Deserialize;

use crate::cache;
use crate::config::BuildlessTransport;
use crate::errors::*;

pub struct BuildlessCache;

// Constants used by the Buildless cache module.
const BUILDLESS_LOCAL: &str = "local.less.build";
const BUILDLESS_LOCAL_PORT_CONTROL: u16 = 42010;
const BUILDLESS_LOCAL_PORT_HTTP: u16 = 42011;
const BUILDLESS_LOCAL_PORT_RESP: u16 = 42012;
const BUILDLESS_GLOBAL: &str = "global.less.build";
const BUILDLESS_GLOBAL_PORT_HTTPS: u16 = 443;
const BUILDLESS_GLOBAL_PORT_RESP: u16 = 6379;
const BUILDLESS_HTTP_PREFIX_GENERIC: &str = "/cache/generic";
const BUILDLESS_HTTP_APIKEY_USERNAME: &str = "apikey";

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct BuildlessAgentEndpoint {
    pub port: u16,
    pub socket: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct BuildlessAgentConfig {
    pub pid: u32,
    pub port: u16,
    pub socket: Option<String>,
    pub control: Option<BuildlessAgentEndpoint>,
}

fn read_agent_config(path: &Path) -> Option<BuildlessAgentConfig> {
    let config: serde_json::Result<BuildlessAgentConfig> =
        serde_json::from_str(path.to_str().unwrap());
    if let Ok(cfg) = config {
        Some(cfg)
    } else {
        None
    }
}

fn validate_port(port: u16, _transport: BuildlessTransport) -> bool {
    port != BUILDLESS_LOCAL_PORT_CONTROL
}

fn build_https(
    use_agent: bool,
    endpoint: &Option<String>,
    apikey: &Option<String>,
    agent: &Option<BuildlessAgentConfig>,
) -> Result<Operator> {
    let mut builder = Webdav::default();

    // setup https endpoint or use global
    if let Some(endpoint) = endpoint {
        builder.endpoint(endpoint);
    } else if use_agent && agent.is_some() {
        let agent_config = agent.as_ref().unwrap();
        let effective_port: u16 = if validate_port(agent_config.port, BuildlessTransport::HTTPS) {
            agent_config.port
        } else {
            BUILDLESS_LOCAL_PORT_HTTP
        };
        builder.endpoint(&format!("http://{BUILDLESS_LOCAL}:{effective_port}"));
    } else {
        builder.endpoint(&format!(
            "https://{BUILDLESS_GLOBAL}:{BUILDLESS_GLOBAL_PORT_HTTPS}"
        ));
    }

    // set default key path
    builder.root(BUILDLESS_HTTP_PREFIX_GENERIC);

    // if we have an explicit API key, use it
    if let Some(apikey) = apikey {
        builder.username(BUILDLESS_HTTP_APIKEY_USERNAME);
        builder.password(apikey);
    }
    let op = Operator::new(builder)?
        .layer(LoggingLayer::default())
        .finish();

    Ok(op)
}

fn build_resp(
    use_agent: bool,
    endpoint: &Option<String>,
    apikey: &Option<String>,
) -> Result<Operator> {
    return if endpoint.is_some() {
        // build with custom redis URL endpoint
        RedisCache::build(endpoint.as_ref().unwrap().as_str())
    } else {
        let protocol: &str;
        let endpoint_target: String;
        if !use_agent {
            protocol = "rediss";
            endpoint_target = format!("{BUILDLESS_LOCAL}:{BUILDLESS_LOCAL_PORT_RESP}");
        } else {
            protocol = "redis"; // do not need TLS wrapping with local agent
            endpoint_target = format!("{BUILDLESS_GLOBAL}:{BUILDLESS_GLOBAL_PORT_RESP}");
        }

        if apikey.is_some() {
            let apikey_value: &String = apikey.as_ref().unwrap();
            let auth = format!("{BUILDLESS_HTTP_APIKEY_USERNAME}:{apikey_value}@");

            // build with apikey or other auth
            RedisCache::build(&[protocol, "://", &auth, &endpoint_target].concat())
        } else {
            // build with implied authorization, or no authorization
            RedisCache::build(&[protocol, "://", &endpoint_target].concat())
        }
    };
}

impl BuildlessCache {
    pub fn build(
        use_agent: &Option<bool>,
        transport_opt: &Option<BuildlessTransport>,
        endpoint: &Option<String>,
        apikey: &Option<String>,
    ) -> Result<Operator> {
        // resolve agent state file path
        let configpath: &str;
        let instancepath: &str;
        if cfg!(windows) {
            configpath = "C:\\ProgramData\\buildless\\buildless-agent.json";
            instancepath = "C:\\ProgramData\\buildless\\buildless-service.id";
        } else if cfg!(unix) {
            // darwin path
            configpath = "/var/tmp/buildless/buildless-agent.json";
            instancepath = "/var/tmp/buildless/buildless-service.id";
        } else {
            unimplemented!("Buildless caching is only supported on macOS, Linux, and Windows.");
        }
        let instance_exists = Path::new(instancepath).exists();
        let agent_config_exists = Path::new(configpath).exists();
        let do_use_agent = use_agent.or(Some(true)).unwrap_or_default() &&  // agent is enabled
            instance_exists &&  // instance is installed on this machine
            agent_config_exists; // agent is running (rendezvous file exists)

        let agent_config: Option<BuildlessAgentConfig> = if do_use_agent {
            read_agent_config(Path::new(configpath))
        } else {
            None
        };
        let transport: &BuildlessTransport = if transport_opt.is_some() {
            transport_opt.as_ref().unwrap()
        } else {
            &BuildlessTransport::AUTO
        };

        match transport {
            BuildlessTransport::AUTO => build_https(do_use_agent, endpoint, apikey, &agent_config),
            BuildlessTransport::HTTPS => build_https(do_use_agent, endpoint, apikey, &agent_config),
            BuildlessTransport::RESP => build_resp(do_use_agent, endpoint, apikey),
            BuildlessTransport::GHA => unimplemented!(
                "GHA protocol is not implemented yet for Buildless SCCache integration."
            ),
        }
    }
}
