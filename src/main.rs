use timpl::*;
use structstruck::strike as strc;

pub mod consts {
    pub mod apps {
        pub const NAMESPACE: &str = "default";
        pub mod frontend {
            pub const PATH: &str = "/";
            pub mod service {
                pub const NAME: &str = "frontend";
                pub const PORT: u16 = 80;
            }
        }
        pub mod backend {
            pub const PATH: &str = "/api";
            pub mod service {
                pub const NAME: &str = "backend";
                pub const PORT: u16 = 80;
            }
        }
    }
    pub mod infrastructure {
        pub mod ingress {

        }
        pub mod monitoring {
            pub const NAMESPACE: &str = "monitoring";
            pub mod dashboard {
                pub const PATH: &str = "/monitoring";
                pub mod service {
                    pub const NAME: &str = "monitoring-dashboard";
                    pub const PORT: u16 = 80;
                }
            }
        }
    }
}

trait Package {
    fn resources(&self, config: &ClusterConfig) -> Vec<String>;
}

#[derive(Clone)]
struct Image {
    registry: String,
    name: String,
    tag: String,
}

strc! {
    struct Deployment {
        config: #[derive(Clone)] struct ClusterConfig {
            stage: #[derive(Clone)] enum {
                Prod,
                Dev,
                Test,
                Local,
                Minimal,
            },
            manifest: #[derive(Clone)] struct {
                apps: #[derive(Clone)] struct {
                    frontend: #[derive(Clone)] struct {
                        enabled: bool,
                        replicas: u16,
                        image: Image,
                    },
                    backend: #[derive(Clone)] struct {
                        enabled: bool,
                        replicas: u16,
                        image: Image,
                    },
                },
                infrastructure: #[derive(Clone)] struct {
                    ingress: #[derive(Clone)] struct {
                        enabled: bool,
                        domains: Vec<String>,
                        tls: bool,
                    },
                    monitoring: #[derive(Clone)] struct {
                        enabled: bool,
                        tracing: bool,
                        logging: bool,
                        metrics: bool,
                        dashboard: bool,
                    },
                },
            },
        },
        packages: Vec<Box<dyn Package>>,
    }
}

impl ToString for Stage {
    fn to_string(&self) -> String {
        match self {
            Stage::Prod => "prod".to_string(),
            Stage::Dev => "dev".to_string(),
            Stage::Test => "test".to_string(),
            Stage::Local => "local".to_string(),
            Stage::Minimal => "minimal".to_string(),
        }
    }
}

#[derive(Clone)]
struct ServiceMapping {
    namespace: String,
    name: String,
    port: u16,
    path: String,
}

struct IngressConfigPackage {}

impl Package for IngressConfigPackage {
    fn resources(&self, config: &ClusterConfig) -> Vec<String> {
        let mut res = vec![];
        config
            .manifest
            .infrastructure
            .ingress
            .domains
            .iter()
            .for_each(|domain| {
                let name = {
                    let mut res = domain.split('.').collect::<Vec<&str>>();
                    res.reverse();
                    res.join("-")
                };

                let mut services: Vec<ServiceMapping> = vec![];

                if config.manifest.apps.frontend.enabled {
                    services.push(ServiceMapping {
                        namespace: consts::apps::NAMESPACE.to_string(),
                        name: consts::apps::frontend::service::NAME.to_string(),
                        port: consts::apps::frontend::service::PORT,
                        path: consts::apps::frontend::PATH.to_string(),
                    })
                }

                if config.manifest.apps.backend.enabled {
                    services.push(ServiceMapping {
                        namespace: consts::apps::NAMESPACE.to_string(),
                        name: consts::apps::backend::service::NAME.to_string(),
                        port: consts::apps::backend::service::PORT,
                        path: consts::apps::backend::PATH.to_string(),
                    })
                }

                if config.manifest.infrastructure.monitoring.dashboard {
                    services.push(ServiceMapping {
                        namespace: consts::infrastructure::monitoring::NAMESPACE.to_string(),
                        name: consts::infrastructure::monitoring::dashboard::service::NAME
                            .to_string(),
                        port: consts::infrastructure::monitoring::dashboard::service::PORT,
                        path: consts::infrastructure::monitoring::dashboard::PATH.to_string(),
                    })
                }

                res.push(timpl! {
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
                        timpl_map_ln!(services.iter(), service, {
                            - path: { service.path }
                              route: { service.namespace }/{ name }-{ service.name }
                        })
                      }
                });
                services.iter().for_each(|service| {
                    res.push(timpl! {
                        apiVersion: k8s.nginx.org/v1
                        kind: VirtualServerRoute
                        metadata:
                          name: { name }-{ service.name }
                          namespace: { service.namespace }
                        spec:
                          host: { domain }
                          upstreams:
                          - name: { service.name }
                            service: { service.name }
                            port: { service.port }
                          subroutes:
                          - path: { service.path }
                            action:
                                pass: { service.name }
                    });
                });
            });
        res
    }
}

fn main() {

    let defalt_manifest = Manifest {
        apps: Apps {
            frontend: Frontend {
                enabled: true,
                replicas: 1,
                image: Image {
                    registry: "docker.io".to_string(),
                    name: "frontend".to_string(),
                    tag: "latest".to_string(),
                },
            },
            backend: Backend {
                enabled: true,
                replicas: 1,
                image: Image {
                    registry: "docker.io".to_string(),
                    name: "backend".to_string(),
                    tag: "latest".to_string(),
                },
            },
        },
        infrastructure: Infrastructure {
            ingress: Ingress {
                enabled: true,
                domains: vec![],
                tls: true,
            },
            monitoring: Monitoring {
                enabled: true,
                tracing: true,
                logging: true,
                metrics: true,
                dashboard: true,
            },
        },
    };

    let prod = ClusterConfig {
        stage: Stage::Prod,
        manifest: {
            let mut manifest = defalt_manifest.clone();
            manifest.apps.frontend.replicas = 3;
            manifest.apps.frontend.image.tag = "prod-latest".to_string();
            manifest.apps.backend.replicas = 3;
            manifest.apps.backend.image.tag = "prod-latest".to_string();
            manifest.infrastructure.ingress.domains = vec!["prod.app.example.com".to_string(), "app.example.com".to_string()];
            manifest
        },
    };

    let dev = ClusterConfig {
        stage: Stage::Dev,
        manifest: {
            let mut manifest = defalt_manifest.clone();
            manifest.apps.frontend.replicas = 2;
            manifest.apps.frontend.image.tag = "dev-latest".to_string();
            manifest.apps.backend.replicas = 2;
            manifest.apps.backend.image.tag = "dev-latest".to_string();
            manifest.infrastructure.ingress.domains = vec!["dev.app.example.com".to_string()];
            manifest
        },
    };

    let test = ClusterConfig {
        stage: Stage::Test,
        manifest: {
            let mut manifest = defalt_manifest.clone();
            manifest.apps.frontend.replicas = 1;
            manifest.apps.frontend.image.tag = "test-latest".to_string();
            manifest.apps.backend.replicas = 1;
            manifest.apps.backend.image.tag = "test-latest".to_string();
            manifest.infrastructure.ingress.domains = vec!["test.app.example.com".to_string()];
            manifest
        },
    };

    let local = ClusterConfig {
        stage: Stage::Local,
        manifest: {
            let mut manifest = defalt_manifest.clone();
            manifest.apps.frontend.replicas = 1;
            manifest.apps.frontend.image.tag = "local-latest".to_string();
            manifest.apps.backend.replicas = 1;
            manifest.apps.backend.image.tag = "local-latest".to_string();
            manifest.infrastructure.ingress.domains = vec!["localhost".to_string()];
            manifest.infrastructure.ingress.tls = false;
            manifest
        },
    };

    let minimal = ClusterConfig {
        stage: Stage::Minimal,
        manifest: {
            let mut manifest = defalt_manifest.clone();
            manifest.apps.frontend.replicas = 1;
            manifest.apps.frontend.image.tag = "minimal-latest".to_string();
            manifest.apps.backend.replicas = 1;
            manifest.apps.backend.image.tag = "minimal-latest".to_string();
            manifest.infrastructure.ingress.domains = vec!["localhost".to_string()];
            manifest.infrastructure.ingress.tls = false;
            manifest.infrastructure.monitoring.enabled = false;
            manifest
        },
    };

    let clusters = vec![
        prod,
        dev,
        test,
        local,
        minimal,
    ];

    let folder = "clusters";
    let _ = std::fs::remove_dir_all(&folder);

    clusters.iter().for_each(|cluster| {
        let folder = format!("{}/{}", folder, cluster.stage.to_string());
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(format!("{}/resources.yaml", folder), {
            let mut res = vec![IngressConfigPackage{}]
                .iter()
                .map(|p| p.resources(cluster))
                .flatten()
                .collect::<Vec<String>>()
                .join("\n---\n");
            res.push('\n');
            res
        })
        .unwrap();
    });
}
