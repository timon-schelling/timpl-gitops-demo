use timpl::*;

use std::str::FromStr;

struct ClusterInfo {
    domains: Vec<String>,
}

enum Cluster {
    Prod,
    Dev,
    Test,
    Local,
}

impl Into<ClusterInfo> for Cluster {
    fn into(self) -> ClusterInfo {
        match self {
            Cluster::Prod => ClusterInfo {
                domains: vec![
                    "prod.app.example.com".to_string(),
                    "app.example.com".to_string(),
                ],
            },
            Cluster::Dev => ClusterInfo {
                domains: vec!["dev.app.example.com".to_string()],
            },
            Cluster::Test => ClusterInfo {
                domains: vec!["test.app.example.com".to_string()],
            },
            Cluster::Local => ClusterInfo {
                domains: vec!["localhost".to_string()],
            },
        }
    }
}

impl FromStr for Cluster {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "prod" => Ok(Cluster::Prod),
            "dev" => Ok(Cluster::Dev),
            "test" => Ok(Cluster::Test),
            "local" => Ok(Cluster::Local),
            _ => Err(format!("Unknown cluster: {}", s)),
        }
    }
}

#[derive(Clone)]
struct Application {
    namespace: String,
    name: String,
    port: u16,
    path: String,
}

fn ingress_for_domain(domain: &str, apps: Vec<Application>) -> String {
    let name = {
        let mut res = domain.split('.').collect::<Vec<&str>>();
        res.reverse();
        res.join("-")
    };

    timpl! {
        apiVersion: k8s.nginx.org/v1
        kind: VirtualServer
        metadata:
          name: { name }
          namespace: ingress
        spec:
          host: { domain }
          tls:
            secret: { name }-tls
          routes:
          {
            timpl_map_ln!(apps.iter(), app, {
                - path: { app.path }
                  route: { app.namespace }/{ name }-{ app.name }
            })
          }
        ---
        {
            timpl_map_ln!(apps.iter(), app, {
                apiVersion: k8s.nginx.org/v1
                kind: VirtualServerRoute
                metadata:
                  name: { name }-{ app.name }
                  namespace: { app.namespace }
                spec:
                  host: { domain }
                  upstreams:
                  - name: { app.name }
                    service: { app.name }
                    port: { app.port }
                  subroutes:
                  - path: { app.path }
                    action:
                      pass: { app.name }
                ---
            })
        }
    }
}

fn template(cluster: Cluster) -> String {
    let cluster_info: ClusterInfo = cluster.into();

    let apps = vec![
        Application {
            namespace: "default".to_string(),
            name: "frontend".to_string(),
            port: 80,
            path: "/".to_string(),
        },
        Application {
            namespace: "default".to_string(),
            name: "backend".to_string(),
            port: 80,
            path: "/api".to_string(),
        },
        Application {
            namespace: "monitoring".to_string(),
            name: "monitoring-dashboard".to_string(),
            port: 80,
            path: "/monitoring".to_string(),
        },
    ];

    timpl_map_ln!(cluster_info.domains.iter(), domain, {
        {
            ingress_for_domain(domain, apps.clone())
        }
    })
}

fn main() {
    let cluster_name = std::env::args().skip(1).next().unwrap_or_default();
    let cluster = Cluster::from_str(&cluster_name).unwrap();
    println!("{}", template(cluster));
}
