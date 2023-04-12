use std::fmt::Display;

use structstruck::strike as strc;

use timpl::*;

pub mod consts {
    pub mod reconsilation {
        pub const INTERVAL: &str = "1m";
    }
    pub mod apps {
        pub const NAMESPACE: &str = "default";
        pub mod frontend {
            pub const PATH: &str = "/";
            pub const NAME: &str = "frontend";
            pub const PORT: u16 = 80;
        }
        pub mod backend {
            pub const PATH: &str = "/api";
            pub const NAME: &str = "backend";
            pub const PORT: u16 = 80;
        }
    }
    pub mod infrastructure {
        pub mod ingress {
            pub const NAMESPACE: &str = "ingress";
            pub const SYSTEM_NAMESPACE: &str = "ingress-system";
            pub const NAME: &str = "ingress";
        }
        pub mod monitoring {
            pub const NAMESPACE: &str = "monitoring";
            pub mod dashboard {
                pub const PATH: &str = "/monitoring";
                pub const NAME: &str = "dashboard";
                pub const PORT: u16 = 80;
            }
        }
    }
}

// pub enum Output {

// }

pub trait Package {
    fn resources(&self, config: &ClusterConfig) -> Vec<String>;
}

strc! {
    #[derive(Clone)]
    pub struct Image {
        pub reference: #[derive(Clone)] pub struct ImageRef {
            pub registry: String,
            pub name: String,
            pub tag: String,
        },
        pub pull_policy: #[derive(Clone)] pub enum {
            Always,
            IfNotPresent,
            Never,
        },
    }
}

impl Display for ImageRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&timpl! { { self.registry }/{ self.name }:{self.tag } })
    }
}

impl Display for PullPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PullPolicy::Always => "Always",
            PullPolicy::IfNotPresent => "IfNotPresent",
            PullPolicy::Never => "Never",
        })
    }
}

#[derive(Clone)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ServiceType::ClusterIP => "ClusterIP",
            ServiceType::NodePort => "NodePort",
            ServiceType::LoadBalancer => "LoadBalancer",
        })
    }
}

strc! {
    pub struct Deployment {
        pub config: #[derive(Clone)] pub struct ClusterConfig {
            pub stage: #[derive(Clone)] pub enum {
                Prod,
                Dev,
                Test,
                Local,
                Minimal,
            },
            pub manifest: #[derive(Clone)] pub struct {
                pub apps: #[derive(Clone)] pub struct {
                    pub frontend: #[derive(Clone)] pub struct {
                        pub enabled: bool,
                        pub replicas: u16,
                        pub image: Image,
                        pub service_type: ServiceType,
                    },
                    pub backend: #[derive(Clone)] pub struct {
                        pub enabled: bool,
                        pub replicas: u16,
                        pub image: Image,
                        pub service_type: ServiceType,
                    },
                },
                pub infrastructure: #[derive(Clone)] pub struct {
                    pub ingress: #[derive(Clone)] pub struct {
                        pub enabled: bool,
                        pub domains: Vec<String>,
                        pub tls: bool,
                    },
                    pub monitoring: #[derive(Clone)] pub struct {
                        pub enabled: bool,
                        pub sources: #[derive(Clone)] pub struct {
                            pub tracing: bool,
                            pub logging: bool,
                            pub metrics: bool,
                        },
                        pub dashboard: #[derive(Clone)] pub struct {
                            pub enabled: bool,
                            pub service_type: ServiceType,
                        },
                    },
                },
            },
        },
        pub packages: Vec<Box<dyn Package>>,
    }
}

impl Deployment {
    pub fn resources(&self) -> Vec<String> {
        self.packages
            .iter()
            .map(|package| package.resources(&self.config))
            .flatten()
            .collect()
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
pub struct ServiceMapping {
    namespace: String,
    name: String,
    port: u16,
    path: String,
}

pub struct FrontendPackage {}

impl Package for FrontendPackage {
    fn resources(&self, config: &ClusterConfig) -> Vec<String> {
        let mut res = vec![];

        res.push(timpl! {
            apiVersion: apps/v1
            kind: Deployment
            metadata:
              namespace: { consts::apps::NAMESPACE }
              name: { consts::apps::frontend::NAME }
            spec:
              replicas: { config.manifest.apps.frontend.replicas }
              selector:
                matchLabels:
                  app: { consts::apps::frontend::NAME }
              template:
                metadata:
                  labels:
                    app: { consts::apps::frontend::NAME }
                spec:
                  containers:
                  - name: { consts::apps::frontend::NAME }
                    image: { config.manifest.apps.frontend.image.reference }
                    imagePullPolicy: { config.manifest.apps.frontend.image.pull_policy }
                    ports:
                    - containerPort: { consts::apps::frontend::PORT }
        });

        res.push(timpl! {
            apiVersion: v1
            kind: Service
            metadata:
              namespace: { consts::apps::NAMESPACE }
              name: { consts::apps::frontend::NAME }
            spec:
              type: { config.manifest.apps.backend.service_type }
              selector:
                app: { consts::apps::frontend::NAME }
              ports:
              - port: { consts::apps::frontend::PORT }
                targetPort: { consts::apps::frontend::PORT }
        });

        res
    }
}

pub struct BackendPackage {}

impl Package for BackendPackage {
    fn resources(&self, config: &ClusterConfig) -> Vec<String> {
        let mut res = vec![];

        res.push(timpl! {
            apiVersion: apps/v1
            kind: Deployment
            metadata:
              namespace: { consts::apps::NAMESPACE }
              name: { consts::apps::backend::NAME }
            spec:
              replicas: { config.manifest.apps.backend.replicas }
              selector:
                matchLabels:
                  app: { consts::apps::backend::NAME }
              template:
                metadata:
                  labels:
                    app: { consts::apps::backend::NAME }
                spec:
                  containers:
                  - name: { consts::apps::backend::NAME }
                    image: { config.manifest.apps.backend.image.reference }
                    imagePullPolicy: { config.manifest.apps.backend.image.pull_policy }
                    ports:
                    - containerPort: { consts::apps::backend::PORT }
        });

        res.push(timpl! {
            apiVersion: v1
            kind: Service
            metadata:
              namespace: { consts::apps::NAMESPACE }
              name: { consts::apps::backend::NAME }
            spec:
              type: { config.manifest.apps.backend.service_type }
              selector:
                app: { consts::apps::backend::NAME }
              ports:
              - port: { consts::apps::backend::PORT }
                targetPort: { consts::apps::backend::PORT }
        });

        res
    }
}

pub struct IngressSystemPackage {}

impl Package for IngressSystemPackage {
    fn resources(&self, _config: &ClusterConfig) -> Vec<String> {
        let mut res = vec![];

        res.push(timpl! {
            apiVersion: source.toolkit.fluxcd.io/v1beta1
            kind: HelmRepository
            metadata:
              name: { consts::infrastructure::ingress::NAME }
              namespace: { consts::infrastructure::ingress::SYSTEM_NAMESPACE }
            spec:
              interval: { consts::reconsilation::INTERVAL }
              url: "https://helm.nginx.com/stable"
        });

        res.push(timpl! {
            apiVersion: helm.toolkit.fluxcd.io/v2beta1
            kind: HelmRelease
            metadata:
              name: { consts::infrastructure::ingress::NAME }
              namespace: { consts::infrastructure::ingress::SYSTEM_NAMESPACE }
            spec:
              chart:
                spec:
                  sourceRef:
                    kind: HelmRepository
                    name: { consts::infrastructure::ingress::NAME }
                  chart: nginx-ingress
                  version: 0.15.2
              interval: { consts::reconsilation::INTERVAL }
              values:
                controller:
                  enableCertManager: true
                  name: { consts::infrastructure::ingress::NAME }
                  enableLatencyMetrics: true
                  config:
                    name: { consts::infrastructure::ingress::NAME }
                  service:
                    name: { consts::infrastructure::ingress::NAME }
                  serviceAccount:
                    name: { consts::infrastructure::ingress::NAME }
                  reportIngressStatus:
                    leaderElectionLockName: { consts::infrastructure::ingress::NAME }-leader-election
                prometheus:
                  create: false
        });

        res
    }
}

pub struct IngressConfigPackage {}

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
                        name: consts::apps::frontend::NAME.to_string(),
                        port: consts::apps::frontend::PORT,
                        path: consts::apps::frontend::PATH.to_string(),
                    })
                }

                if config.manifest.apps.backend.enabled {
                    services.push(ServiceMapping {
                        namespace: consts::apps::NAMESPACE.to_string(),
                        name: consts::apps::backend::NAME.to_string(),
                        port: consts::apps::backend::PORT,
                        path: consts::apps::backend::PATH.to_string(),
                    })
                }

                if config.manifest.infrastructure.monitoring.enabled
                    && config.manifest.infrastructure.monitoring.dashboard.enabled
                {
                    services.push(ServiceMapping {
                        namespace: consts::infrastructure::monitoring::NAMESPACE.to_string(),
                        name: consts::infrastructure::monitoring::dashboard::NAME.to_string(),
                        port: consts::infrastructure::monitoring::dashboard::PORT,
                        path: consts::infrastructure::monitoring::dashboard::PATH.to_string(),
                    })
                }

                res.push(timpl! {
                    apiVersion: k8s.nginx.org/v1
                    kind: VirtualServer
                    metadata:
                      name: { name }
                      namespace: { consts::infrastructure::ingress::NAMESPACE }
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
