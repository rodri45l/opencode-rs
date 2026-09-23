//! Port of packages/console/core/test/model.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/core/src/model.ts (`ZenData.validate`):
//! a `cost200K` block without an explicit threshold defaults to 200_000, an
//! explicit threshold is preserved, and the `systemone` provider format is
//! accepted.
//! Re-derived: the parsed-JSON input is represented as typed model/provider
//! values rather than untyped `JSON.parse` output.

#[allow(dead_code)]
mod model {
    use std::collections::BTreeMap;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: console zen model validation not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Cost {
        pub input: i64,
        pub output: i64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Cost200K {
        pub input: i64,
        pub output: i64,
        pub threshold: Option<i64>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ResolvedCost200K {
        pub input: i64,
        pub output: i64,
        pub threshold: i64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProviderRef {
        pub id: String,
        pub model: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RawZenModel {
        pub name: String,
        pub cost: Cost,
        pub cost_multiplier: i64,
        pub cost200k: Option<Cost200K>,
        pub providers: Vec<ProviderRef>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ZenModel {
        pub name: String,
        pub cost: Cost,
        pub cost_multiplier: i64,
        pub cost200k: Option<ResolvedCost200K>,
        pub providers: Vec<ProviderRef>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Provider {
        pub api: String,
        pub api_key: String,
        pub format: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ZenData {
        pub zen_models: BTreeMap<String, ZenModel>,
        pub providers: BTreeMap<String, Provider>,
    }

    impl ZenData {
        pub fn validate(
            _zen_models: BTreeMap<String, RawZenModel>,
            _providers: BTreeMap<String, Provider>,
        ) -> PortResult<ZenData> {
            Err(NotImplemented(NOTE))
        }
    }
}

use std::collections::BTreeMap;

use model::{Cost, Cost200K, Provider, ProviderRef, RawZenModel, ZenData, NOTE};

fn base_model(cost200k: Option<Cost200K>) -> RawZenModel {
    RawZenModel {
        name: "GPT-5.6 Sol".to_string(),
        cost: Cost {
            input: 2,
            output: 10,
        },
        cost_multiplier: 1,
        cost200k,
        providers: vec![ProviderRef {
            id: "openai".to_string(),
            model: "gpt-5.6-sol".to_string(),
        }],
    }
}

fn zen_models(model: RawZenModel) -> BTreeMap<String, RawZenModel> {
    let mut map = BTreeMap::new();
    map.insert("gpt-5.6-sol".to_string(), model);
    map
}

fn openai_provider() -> BTreeMap<String, Provider> {
    let mut map = BTreeMap::new();
    map.insert(
        "openai".to_string(),
        Provider {
            api: "https://api.openai.com/v1".to_string(),
            api_key: "test".to_string(),
            format: None,
        },
    );
    map
}

#[test]
#[ignore = "porting: console zen model validation not implemented"]
fn defaults_to_200_000_when_not_configured() {
    let data = ZenData::validate(
        zen_models(base_model(Some(Cost200K {
            input: 4,
            output: 15,
            threshold: None,
        }))),
        openai_provider(),
    )
    .expect(NOTE);

    let model = &data.zen_models["gpt-5.6-sol"];
    assert_eq!(
        model.cost200k.as_ref().map(|cost| cost.threshold),
        Some(200_000)
    );
}

#[test]
#[ignore = "porting: console zen model validation not implemented"]
fn accepts_an_explicit_272_000_threshold() {
    let data = ZenData::validate(
        zen_models(base_model(Some(Cost200K {
            input: 4,
            output: 15,
            threshold: Some(272_000),
        }))),
        openai_provider(),
    )
    .expect(NOTE);

    let model = &data.zen_models["gpt-5.6-sol"];
    assert_eq!(
        model.cost200k.as_ref().map(|cost| cost.threshold),
        Some(272_000)
    );
}

#[test]
#[ignore = "porting: console zen model validation not implemented"]
fn accepts_the_systemone_provider_format() {
    let mut providers = BTreeMap::new();
    providers.insert(
        "systemone".to_string(),
        Provider {
            api: "https://api.typesafe.ai/v1".to_string(),
            api_key: "test".to_string(),
            format: Some("systemone".to_string()),
        },
    );

    let data = ZenData::validate(zen_models(base_model(None)), providers).expect(NOTE);
    assert_eq!(
        data.providers["systemone"].format.as_deref(),
        Some("systemone")
    );
}
