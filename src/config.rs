use timpl_gitops_demo_gen_base::*;

fn default_manifest() -> Manifest {
    Manifest {
        apps: Apps {
            frontend: Frontend {
                enabled: true,
                replicas: 1,
                image: Image {
                    reference: ImageRef {
                        registry: "cr.example.com".to_string(),
                        name: "frontend".to_string(),
                        tag: "latest".to_string(),
                    },
                    pull_policy: PullPolicy::Always,
                },
                service_type: ServiceType::ClusterIP,
            },
            backend: Backend {
                enabled: true,
                replicas: 1,
                image: Image {
                    reference: ImageRef {
                        registry: "cr.example.com".to_string(),
                        name: "backend".to_string(),
                        tag: "latest".to_string(),
                    },
                    pull_policy: PullPolicy::Always,
                },
                service_type: ServiceType::ClusterIP,
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
                sources: Sources {
                    tracing: true,
                    logging: true,
                    metrics: true,
                },
                dashboard: Dashboard {
                    enabled: true,
                    service_type: ServiceType::ClusterIP,
                },
            },
        },
    }
}

pub(super) mod clusters {
    use super::*;

    pub(super) fn prod() -> ClusterConfig {
        ClusterConfig {
            stage: Stage::Prod,
            manifest: {
                let mut manifest = default_manifest();
                manifest.apps.frontend.replicas = 3;
                manifest.apps.frontend.image.reference.tag = "prod-latest".to_string();
                manifest.apps.backend.replicas = 3;
                manifest.apps.backend.image.reference.tag = "prod-latest".to_string();
                manifest.infrastructure.ingress.domains = vec![
                    "prod.app.example.com".to_string(),
                    "app.example.com".to_string(),
                ];
                manifest
            },
        }
    }

    pub(super) fn dev() -> ClusterConfig {
        ClusterConfig {
            stage: Stage::Dev,
            manifest: {
                let mut manifest = default_manifest();
                manifest.apps.frontend.replicas = 2;
                manifest.apps.frontend.image.reference.tag = "dev-latest".to_string();
                manifest.apps.backend.replicas = 2;
                manifest.apps.backend.image.reference.tag = "dev-latest".to_string();
                manifest.infrastructure.ingress.domains = vec!["dev.app.example.com".to_string()];
                manifest
            },
        }
    }

    pub(super) fn test() -> ClusterConfig {
        ClusterConfig {
            stage: Stage::Test,
            manifest: {
                let mut manifest = default_manifest();
                manifest.apps.frontend.replicas = 1;
                manifest.apps.frontend.image.reference.tag = "test-latest".to_string();
                manifest.apps.backend.replicas = 1;
                manifest.apps.backend.image.reference.tag = "test-latest".to_string();
                manifest.infrastructure.ingress.domains = vec!["test.app.example.com".to_string()];
                manifest
            },
        }
    }

    pub(super) fn local() -> ClusterConfig {
        ClusterConfig {
            stage: Stage::Local,
            manifest: {
                let mut manifest = default_manifest();
                manifest.apps.frontend.replicas = 1;
                manifest.apps.frontend.image.reference.tag = "local-latest".to_string();
                manifest.apps.backend.replicas = 1;
                manifest.apps.backend.image.reference.tag = "local-latest".to_string();
                manifest.infrastructure.ingress.domains = vec!["localhost".to_string()];
                manifest.infrastructure.ingress.tls = false;
                manifest
            },
        }
    }

    pub(super) fn minimal() -> ClusterConfig {
        ClusterConfig {
            stage: Stage::Minimal,
            manifest: {
                let mut manifest = default_manifest();
                manifest.apps.frontend.replicas = 1;
                manifest.apps.frontend.image.reference.tag = "minimal-latest".to_string();
                manifest.apps.backend.replicas = 1;
                manifest.apps.backend.image.reference.tag = "minimal-latest".to_string();
                manifest.infrastructure.ingress.domains = vec!["localhost".to_string()];
                manifest.infrastructure.ingress.tls = false;
                manifest.infrastructure.monitoring.enabled = false;
                manifest
            },
        }
    }
}

fn clusters() -> Vec<ClusterConfig> {
    use clusters::*;
    vec![prod(), dev(), test(), local(), minimal()]
}

fn packages() -> Vec<Box<dyn Package>> {
    vec![
        Box::new(FrontendPackage {}),
        Box::new(BackendPackage {}),
        Box::new(IngressSystemPackage {}),
        Box::new(IngressConfigPackage {}),
    ]
}

pub fn deployments() -> Vec<Deployment> {
    clusters()
        .iter()
        .map(|config| Deployment {
            config: config.clone(),
            packages: packages(),
        })
        .collect()
}
