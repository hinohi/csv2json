use serde::ser::SerializeMap;

#[derive(Debug)]
pub struct Map<'a> {
    headers: &'a [&'a str],
    values: &'a [&'a str],
}

impl<'a> Map<'a> {
    pub fn new(headers: &'a [&'a str], values: &'a [&'a str]) -> Map<'a> {
        Map { headers, values }
    }
}

impl<'a> serde::Serialize for Map<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(Some(self.headers.len()))?;
        for (key, value) in self.headers.iter().zip(self.values.iter()) {
            s.serialize_entry(key, value)?;
        }
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let map = Map::new(&["a", "b", "a"], &["1", "2", "3"]);
        assert_eq!(
            serde_json::to_string(&map).unwrap(),
            r#"{"a":"1","b":"2","a":"3"}"#
        )
    }
}
