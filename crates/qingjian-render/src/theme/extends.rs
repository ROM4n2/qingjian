//! 主题继承：`"extends": "qingjian"` 以那个内置主题为底，本文件写的覆盖上去。在 JSON 层合并：
//! 对象逐键合并（`variables` 只写要改的几个颜色即可），数组与标量整个替换（`children` 写了就是整组换掉）。

use serde_json::{Map, Value};

use super::ThemeError;

/// 继承链最多几层，挡住互相继承。
const MAX_DEPTH: usize = 4;

/// 展开 `extends`，返回合并后的整份主题。`base` 按 id 给出内置主题的源文件。
pub(super) fn resolve(
    json: &str,
    base: &dyn Fn(&str) -> Option<&'static str>,
) -> Result<Value, ThemeError> {
    resolve_value(serde_json::from_str(json)?, base, 0)
}

fn resolve_value(
    mut value: Value,
    base: &dyn Fn(&str) -> Option<&'static str>,
    depth: usize,
) -> Result<Value, ThemeError> {
    let Some(id) = value
        .as_object_mut()
        .and_then(|object| object.remove("extends"))
    else {
        return Ok(value);
    };
    let Value::String(id) = id else {
        return Err(ThemeError::UnknownBase { id: id.to_string() });
    };
    if depth >= MAX_DEPTH {
        return Err(ThemeError::TooDeep { id });
    }
    let source = base(&id).ok_or_else(|| ThemeError::UnknownBase { id: id.clone() })?;
    let mut merged = resolve_value(serde_json::from_str(source)?, base, depth + 1)?;
    merge(&mut merged, value);
    Ok(merged)
}

/// 把 `over` 合并进 `base`：两边都是对象就逐键递归，否则整个替换。
fn merge(base: &mut Value, over: Value) {
    match (base, over) {
        (Value::Object(base), Value::Object(over)) => merge_objects(base, over),
        (base, over) => *base = over,
    }
}

fn merge_objects(base: &mut Map<String, Value>, over: Map<String, Value>) {
    for (key, value) in over {
        match base.get_mut(&key) {
            Some(existing) => merge(existing, value),
            None => {
                base.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r##"{ "a": { "x": 1, "y": [1, 2] }, "b": "keep" }"##;

    fn base(id: &str) -> Option<&'static str> {
        match id {
            "base" => Some(BASE),
            "loop" => Some(r#"{ "extends": "loop" }"#),
            _ => None,
        }
    }

    #[test]
    fn merges_objects_and_replaces_arrays() {
        let merged = resolve(r#"{ "extends": "base", "a": { "y": [3] }, "c": 1 }"#, &base).unwrap();
        assert_eq!(
            merged,
            serde_json::json!({ "a": { "x": 1, "y": [3] }, "b": "keep", "c": 1 })
        );
    }

    #[test]
    fn rejects_unknown_and_cyclic_bases() {
        assert!(matches!(
            resolve(r#"{ "extends": "nope" }"#, &base),
            Err(ThemeError::UnknownBase { .. })
        ));
        assert!(matches!(
            resolve(r#"{ "extends": "loop" }"#, &base),
            Err(ThemeError::TooDeep { .. })
        ));
    }
}
