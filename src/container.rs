use form_urlencoded;
use httparse::{Response, EMPTY_HEADER};
use serde::Deserialize;
use serde_yaml;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
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
    container_id: String,
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
    ContainerProcessNotFound,
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
            Error::ContainerProcessNotFound => write!(f, "container process not found"),
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
            container_id: id.to_string(),
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
    pub fn update_pid(&mut self) -> Result<u32, Box<dyn std::error::Error>> {
        self.pid = get_pid_from_container_id(self.container_id())?;
        Ok(self.pid)
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
    pub fn container_id(&self) -> &str {
        &self.container_id
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

    pub fn restart(&self) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.container_id();
        request_docker_api("POST", &format!("/containers/{id}/restart", id = id), None)?;

        Ok(())
    }

    pub fn restart_from_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let id = Self::convert_name_to_id(name)?;
        request_docker_api("POST", &format!("/containers/{id}/restart", id = id), None)?;

        Ok(())
    }

    pub fn convert_injesh_name_to_docker_id(
        injesh_container_name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // read `settings.yaml`
        let settings_file_path = std::path::Path::new(crate::user::User::new()?.containers())
            .join(injesh_container_name)
            .join("settings.yaml");

        // extract `docker_container_id` from yaml
        let settings_yaml_str = std::fs::read_to_string(settings_file_path)?;
        let settings_yaml: SettingsYaml = serde_yaml::from_str(&settings_yaml_str)?;
        let docker_container_id = settings_yaml.docker_container_id;

        Ok(docker_container_id)
    }
}

#[derive(Debug, Deserialize)]
pub struct SettingsYaml {
    pub docker_container_id: String,
    pub shell: String,
    pub commands: Vec<String>,
}

fn get_pid_from_container_id(target_container_id: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let pid_list = std::fs::read_dir("/proc")?;
    // filter only pid (excluding /proc/uptime .. etc.)
    let pid_list: Vec<u32> = pid_list
        .filter_map(|entry| {
            entry
                .ok()?
                .file_name()
                .into_string()
                .unwrap_or("".to_string())
                .parse::<u32>()
                .ok()
        })
        .collect();

    // reverse pid_list
    // We can guess the Docker process is being behind in PID order in most cases.
    let pid_list: Vec<u32> = pid_list.iter().rev().cloned().collect();

    let pid = search_pid_linear(pid_list, target_container_id)?;

    if !valid_pid_is_container(pid)? {
        Err(Error::InvalidPid)?
    }

    Ok(pid)
}

fn search_pid_linear(
    pid_list: Vec<u32>,
    container_id: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    // linear search
    for pid in pid_list {
        // `/proc/{pid}/cmdline` contains process name and arguments.
        let cmdline: String = std::fs::read_to_string(&format!("/proc/{pid}/cmdline", pid = pid))?;
        // docker process arg `-id\0<ID>` is the key to find the container.
        if cmdline.contains(&format!("-id\0{}", container_id)) {
            // `/proc/{pid}/task/{pid}/children` contains child pids.
            let child_pid_list =
                std::fs::read_to_string(&format!("/proc/{pid}/task/{pid}/children", pid = pid))?;
            // what we need is the first child pid.
            // if child pid is single, trailing whitespace delimiter is still used, so can split it.
            let first_child_pid = child_pid_list
                .split_once(' ')
                .ok_or(Error::ContainerProcessNotFound)?
                .0
                .parse::<u32>()?;

            return Ok(first_child_pid);
        }
    }

    Err(Error::ContainerProcessNotFound)?
}

/// valid: Ok(true)
///
/// invalid: Ok(false)
///
/// error: Err
fn valid_pid_is_container(pid: u32) -> Result<bool, Box<dyn std::error::Error>> {
    // `/proc/{pid}/cmdline` contains many arguments.
    let pid_string = std::fs::read_to_string(&format!("/proc/{pid}/stat", pid = pid))?;
    // the parent process id is the forth argument.
    let parent_pid = pid_string
        .split_whitespace()
        .nth(3)
        .ok_or(Error::InvalidPid)?
        .parse::<u32>()?;

    // docker process contains `moby` in its process name.
    let parent_pid_cmdline =
        std::fs::read_to_string(&format!("/proc/{pid}/cmdline", pid = parent_pid))?;
    if !parent_pid_cmdline.contains("moby") {
        return Ok(false);
    }

    Ok(true)
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

    let header_and_body = response
        .split_once("\r\n\r\n")
        .ok_or(Error::InvalidResponse)?;
    let header = header_and_body.0.trim().to_string();

    // Exclude response header
    let mut response_body = header_and_body.1.trim().to_string();

    // Api error catch
    // println!("respo: ```{:?}```", response);

    // check response header
    // if returned other than 200
    // return Error
    let mut headers = [EMPTY_HEADER; 100];
    let mut res = Response::new(&mut headers[..]);
    res.parse(header.as_ref())?;
    match res.code {
        Some(code) => {
            match DockerdResponse::new(code) {
                DockerdResponse::NoError => { /* do nothing */ }
                _ => {
                    // Example: {"message":"No such container"}
                    if let Some(message) = catch_error_message(&response)? {
                        Err(Error::ApiResponseError(message))?
                    }
                    Err(Error::InvalidResponse)?
                }
            }
        }
        None => Err(Error::InvalidResponse)?,
    };

    // Case of inspect, response contains some string like:
    // 1522\r\n{"Id":...}\n\r\n0
    let newline_number = response_body.match_indices('\n').count();
    if newline_number > 1 {
        response_body = response_body.split("\n").collect::<Vec<&str>>()[1]
            .trim()
            .to_string();
    }

    // wrap with a brath
    // println!("res: ```{}```", response_body);
    let response_body = format!(
        r#"{{"containers":{response_body}}}"#,
        response_body = response_body
    );

    Ok(response_body)
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

pub enum DockerdResponse {
    // 204 or 200
    NoError,
    // 404
    NoSuchContainer,
    // 500
    ServerError,
    NotFound,
}

impl DockerdResponse {
    fn new(response_id: u16) -> DockerdResponse {
        match response_id {
            200 => DockerdResponse::NoError,
            204 => DockerdResponse::NoError,
            404 => DockerdResponse::NoSuchContainer,
            500 => DockerdResponse::ServerError,
            _ => DockerdResponse::NotFound,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_convert_injesh_name_to_docker_id() {
        let name = "tes";
        let id = Container::convert_injesh_name_to_docker_id(name).unwrap();
        println!("id: {}", id);
    }
}
