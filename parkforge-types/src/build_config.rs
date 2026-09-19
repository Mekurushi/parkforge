use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildConfig {
    values: BTreeMap<String, BuildConfigValue>,
}

impl BuildConfig {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn values(&self) -> &BTreeMap<String, BuildConfigValue> {
        &self.values
    }

    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: BuildConfigValue,
    ) -> Option<BuildConfigValue> {
        self.values.insert(name.into(), value)
    }
}

impl FromIterator<(String, BuildConfigValue)> for BuildConfig {
    fn from_iter<T: IntoIterator<Item = (String, BuildConfigValue)>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuildConfigValue {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(String),
}
