use serde::Serialize;

#[derive(Debug, Clone)]
pub struct RepoConfig {
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoPackage {
    pub name: String,
    pub arch: String,
    pub epoch: u32,
    pub version: String,
    pub release: String,
    pub provides: Vec<RepoProvide>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoProvide {
    pub name: String,
    pub flags: Option<String>,
    pub epoch: Option<u32>,
    pub version: Option<String>,
    pub release: Option<String>,
}

impl RepoConfig {
    pub fn centos_stream_10() -> Vec<Self> {
        vec![
            RepoConfig {
                name: "BaseOS".into(),
                base_url: "https://mirror.stream.centos.org/10-stream/BaseOS/x86_64/os".into(),
            },
            RepoConfig {
                name: "AppStream".into(),
                base_url: "https://mirror.stream.centos.org/10-stream/AppStream/x86_64/os".into(),
            },
            RepoConfig {
                name: "CRB".into(),
                base_url: "https://mirror.stream.centos.org/10-stream/CRB/x86_64/os".into(),
            },
        ]
    }

    pub fn epel10() -> Self {
        RepoConfig {
            name: "EPEL10".into(),
            base_url: "https://dl.fedoraproject.org/pub/epel/10/Everything/x86_64".into(),
        }
    }
}
