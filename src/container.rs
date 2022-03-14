use form_urlencoded;
use serde::Deserialize;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::{error, fmt, path};
// debug
// use std::collections::HashMap;
// use std::ffi::OsStr;

#[derive(Debug, Deserialize)]
struct DockerListApiResponse {
    containers: Vec<DockerContainerList>,
}
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DockerContainerList {
    Id: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DockerInspectApiResponse {
    containers: DockerContainerInspect,
}
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DockerContainerInspect {
    GraphDriver: DockerGraphDriver,
}
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DockerGraphDriver {
    Name: String,
    Data: DockerGraphDriverData,
}
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct DockerGraphDriverData {
    LowerDir: path::PathBuf,
    UpperDir: path::PathBuf,
    MergedDir: path::PathBuf,
    WorkDir: path::PathBuf,
}

#[derive(Debug)]
pub struct Container {
    pid: u32,
    lowerdir: path::PathBuf,
    upperdir: path::PathBuf,
    mergeddir: path::PathBuf,
    workdir: path::PathBuf,
}

#[derive(Debug)]
pub enum Error {
    NotInitialized,
    InvalidPid,
    InvalidParameter,
    InvalidResponse,
    ContainerNotFound,
    GraphDriverNotOverlay2,
    GraphDriverPathNotFound,
    ApiResponseError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "Not initialized"),
            Error::InvalidPid => write!(f, "Invalid PID"),
            Error::InvalidParameter => write!(f, "Invalid Parameter"),
            Error::InvalidResponse => write!(f, "Invalid Response"),
            Error::ContainerNotFound => write!(f, "Container Not Found"),
            Error::GraphDriverNotOverlay2 => write!(f, "GraphDriver is not overlay2"),
            Error::GraphDriverPathNotFound => write!(f, "GraphDriver path not found"),
            Error::ApiResponseError(message) => {
                write!(f, "API response error with message: {}", message)
            }
        }
    }
}

impl error::Error for Error {}

impl Container {
    pub fn new(name_or_id: &str) -> Result<Container, Box<dyn std::error::Error>> {
        let id = name_or_id;
        let pid = match get_pid_from_container_id(id) {
            Ok(pid) => {
                // println!("\n\nYeah it's ID. {}\n\n", pid);
                pid
            }
            Err(_) => {
                // println!("\n\n\nnot id.\n\n\n");
                let may_name = name_or_id;
                let id = Self::convert_name_to_id(may_name)?;
                get_pid_from_container_id(&id)?
            }
        };

        // println!("------------");
        let docker_info: DockerInspectApiResponse = serde_json::from_str(&request_docker_api(
            "GET",
            &format!("/containers/{id}/json", id = id),
            None,
        )?)?;
        // println!("\n\ndockerinfo: ```{:?}```", docker_info);
        if docker_info.containers.GraphDriver.Name != "overlay2" {
            Err(Error::GraphDriverNotOverlay2)?
        }
        let graph_driver_data = docker_info.containers.GraphDriver.Data;

        Ok(Container {
            pid,
            lowerdir: graph_driver_data.LowerDir,
            upperdir: graph_driver_data.UpperDir,
            mergeddir: graph_driver_data.MergedDir,
            workdir: graph_driver_data.WorkDir,
        })
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn lowerdir(&self) -> &std::path::PathBuf {
        &self.lowerdir
    }
    pub fn mergeddir(&self) -> &std::path::PathBuf {
        &self.mergeddir
    }
    pub fn upperdir(&self) -> &std::path::PathBuf {
        &self.upperdir
    }
    pub fn workdir(&self) -> &std::path::PathBuf {
        &self.workdir
    }
    pub fn convert_name_to_id(name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // as commandline:
        // curl --unix-socket /var/run/docker.sock -X GET "http://localhost/containers/json?all=true&filters=$( python3 -c 'import urllib.parse; print( urllib.parse.quote("""{"name": ["beautiful_curran"]}""") )' )"
        let docker_info: DockerListApiResponse = serde_json::from_str(&request_docker_api(
            "GET",
            "/containers/json",
            Some(&format!(r#"all=true&filters={{"name": ["{}"]}}"#, name)),
        )?)?;

        if docker_info.containers.is_empty() {
            Err(Error::ContainerNotFound)?
        }
        if docker_info.containers.len() > 1 {
            Err(Error::ContainerNotFound)?
        }

        let mut id = docker_info.containers[0].Id.to_string();
        id.truncate(12);

        Ok(id)
    }
}

fn get_pid_from_container_id(target_container_id: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // TODO: Eliminate dependency on shell command.
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&format!("ps --ppid $(ps ax -o pid= -o args= | grep 'moby' | grep '\\-id {target_container_id}' | awk '$0=$0'1 | head -1) -o pid=", target_container_id = target_container_id));
    let out = cmd.output()?;
    // debug
    // let out: Vec<&OsStr> = cmd.get_args().collect();
    let pid = String::from_utf8(out.stdout)?.trim().parse::<u32>()?;

    valid_pid_is_container(pid)?;

    Ok(pid)
}

fn valid_pid_is_container(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Eliminate dependency on shell command.
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(format!("ps -p $(ps -p {pid} -o ppid=) -o args=", pid = pid));
    let out = cmd.output()?;
    // check if `out` contains `moby`
    let stdout = String::from_utf8(out.stdout)?;
    if !stdout.contains("moby") {
        Err(Error::InvalidPid)?
    }

    Ok(())
}

fn encode_request_path(path: &str, params: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut encoded_params = form_urlencoded::Serializer::new(String::new());
    // valid parameter contains '=' character.
    if !params.contains('=') {
        // eprintln!("Error happend");
        Err(Error::InvalidParameter)?
    }

    params.split('&').for_each(|param| {
        let k_v = param.split_once('=').unwrap();
        encoded_params.append_pair(k_v.0, k_v.1);
    });
    let encoded_params = encoded_params.finish();
    let request_path = format!(
        "{path}?{encoded_params}",
        path = path,
        encoded_params = encoded_params
    );

    Ok(request_path)
}

/// Hitting Docker API Interface.  
/// See API reference: https://docs.docker.com/engine/api/latest/
/// 
/// # Params:
/// 
/// - method: GET, POST, etc
/// - path: "/containers/json", etc
/// - parameter: Some("filters={"name":"hoge"}"), None, etc
/// 
/// # response:
/// 
/// type: String  
/// receiving json format:
/// 
/// ```ignore
/// {"containers": {API response body}}
/// ```
/// 
/// So, We must receive values using `containers` key.  
/// Deserializing by `serde_json::from_str()` is useful.
/// 
/// # Example:
/// 
/// Receiveing Array Value
/// 
/// ```ignore
/// struct ListResponse {
///     // Vector.
///     containers: Vec<ListValues>
/// }
///
/// ... snip ...
///
/// let docker_info: ListResponse = serde_json::from_str(&request_docker_api("GET", "/containers/json", Some(&format!(r#"all=true&filters={{"name": ["{}"]}}"#, name)))?)?;
/// ```
/// 
/// Receiveing Object Value
/// 
/// ```ignore
/// struct InspectResponse {
///     // Not a Vector.
///     containers: InspectValues
/// }
///
/// ... snip ...
///
/// let docker_info: InspectResponse = serde_json::from_str(&request_docker_api("GET", &format!("/containers/{id}/json", id = id), None)?)?;
/// ```
fn request_docker_api(
    method: &str,
    path: &str,
    parameter: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let docker_sock = std::path::Path::new("/var/run/docker.sock");

    // Generate request body
    let mut request_path = String::new();
    if let Some(parameter) = parameter {
        request_path = encode_request_path(path, parameter)?;
    } else {
        request_path = path.to_string();
    }
    let request = format!(
        "{method} {request_path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        method = method,
        request_path = request_path
    );

    // println!("request: `{:?}`", request);
    // Request to docker api
    let mut stream = UnixStream::connect(docker_sock)?;
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    drop(stream);

    // Exclude response header
    let mut response = response
        .split_once("\r\n\r\n")
        .ok_or(Error::InvalidResponse)?
        .1
        .trim()
        .to_string();

    // Api error catch
    // println!("respo: ```{:?}```", response);
    if response.is_empty() || response == "[]" {
        Err(Error::ContainerNotFound)?
    }
    // Example: {"message":"No such container"}
    if let Some(message) = catch_error_message(&response)? {
        Err(Error::ApiResponseError(message))?
    }

    // Case of inspect, response contains some string like:
    // 1522\r\n{"Id":...}\n\r\n0
    let newline_number = response.match_indices('\n').count();
    if newline_number > 1 {
        response = response.split("\n").collect::<Vec<&str>>()[1]
            .trim()
            .to_string();
    } else {
        response = response;
    }

    // wrap with a brath
    // println!("res: ```{}```", response);
    let response = format!(r#"{{"containers":{response}}}"#, response = response);

    Ok(response)
}

fn catch_error_message(response: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if !response.contains(r#""message":"#) {
        return Ok(None);
    }

    #[derive(Debug, Deserialize)]
    struct Message {
        message: String,
    }
    let message: Message = serde_json::from_str(&response)?;

    Ok(Some(message.message))
}
