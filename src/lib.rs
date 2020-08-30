use std::collections::HashMap;
use std::fmt;




// ============================================================================
#[derive(Clone)]
pub enum Value {
    B(bool),
    I(i64),
    F(f64),
    S(String),
}

impl From<bool> for Value { fn from(a: bool) -> Self { Value::B(a) } }
impl From<i64>  for Value { fn from(a: i64)  -> Self { Value::I(a) } }
impl From<f64>  for Value { fn from(a: f64)  -> Self { Value::F(a) } }
impl From<&str> for Value { fn from(a: &str) -> Self { Value::S(a.into()) } }

impl Value {
    pub fn same_kind_as(&self, other: &Value) -> bool {
       match (&self, &other) {
           (Value::B(_), Value::B(_)) => true,
           (Value::I(_), Value::I(_)) => true,
           (Value::F(_), Value::F(_)) => true,
           (Value::S(_), Value::S(_)) => true,
           _ => false,
       }
    }
    pub fn as_bool  (&self) -> bool   { match self { Value::B(x) => x.clone(), _ => panic!() } }
    pub fn as_int   (&self) -> i64    { match self { Value::I(x) => x.clone(), _ => panic!() } }
    pub fn as_float (&self) -> f64    { match self { Value::F(x) => x.clone(), _ => panic!() } }
    pub fn as_string(&self) -> String { match self { Value::S(x) => x.clone(), _ => panic!() } }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::B(x) => x.fmt(f),
            Value::I(x) => x.fmt(f),
            Value::F(x) => x.fmt(f),
            Value::S(x) => x.fmt(f),
        }
    }
}




// ============================================================================
#[derive(Clone)]
pub struct Parameter {
    pub value: Value,
    pub about: String,
}




// ============================================================================
#[derive(Debug)]
pub struct ConfigError
{
    key: String,
    why: String,
}

impl ConfigError {
    pub fn new(key: &str, why: &str) -> ConfigError {
        ConfigError{key: key.into(), why: why.into()}
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "config key '{}' {}", self.key, self.why)
    }
}

impl std::error::Error for ConfigError {}




// ============================================================================
pub struct Form {
    parameter_map: HashMap::<String, Parameter>
}




// ============================================================================
impl Form {
    pub fn new() -> Form {
        Form{parameter_map: HashMap::new()}
    }

    pub fn item<T: Into<Value>>(&self, key: &str, default: T, about: &str) -> Self {
        let mut parameter_map = self.parameter_map.clone();
        parameter_map.insert(key.into(), Parameter{value: default.into(), about: about.into()});
        Form{parameter_map: parameter_map}
    }

    pub fn merge_value_map(&self, items: &HashMap<String, Value>) -> Result<Self, ConfigError> {
        let mut parameter_map = self.parameter_map.clone();
        for (key, new_value) in items {
            if ! parameter_map
                .get(key)
                .map(|p| p.value.clone())
                .ok_or(ConfigError::new(key, "is not a valid key"))?
                .same_kind_as(&new_value) {
                    return Err(ConfigError::new(key, "has the wrong type"))
                }
            parameter_map.entry(key.into()).and_modify(|p| p.value = new_value.clone());
        }
        Ok(Form{parameter_map: parameter_map})
    }

    pub fn merge_string_map(&self, dict: HashMap<String, String>) -> Result<Self, ConfigError> {
        use Value::*;
        let mut parameter_map = self.parameter_map.clone();
        for (k, v) in &dict {
            let parameter = self.parameter_map.get(k).ok_or(ConfigError::new(&k, "is not a valid key"))?;
            let new_value = match parameter.value {
                B(_) => v.parse().map(|x| B(x)).map_err(|_| ConfigError::new(k, "is a badly formed bool")),
                I(_) => v.parse().map(|x| I(x)).map_err(|_| ConfigError::new(k, "is a badly formed int")),
                F(_) => v.parse().map(|x| F(x)).map_err(|_| ConfigError::new(k, "is a badly formed float")),
                S(_) => v.parse().map(|x| S(x)).map_err(|_| ConfigError::new(k, "is a badly formed string")),
            }?;
            parameter_map.entry(k.into()).and_modify(|p| p.value = new_value);
        }
        Ok(Form{parameter_map: parameter_map})
    }

    pub fn get(&self, key: &str) -> &Value {
        &self.parameter_map.get(key.into()).unwrap().value
    }
}




// ============================================================================
impl<'a> IntoIterator for &'a Form {
    type Item     = <&'a HashMap<String, Parameter> as IntoIterator>::Item;
    type IntoIter = <&'a HashMap<String, Parameter> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.parameter_map.iter()
    }
}




// ============================================================================
pub fn to_string_map_from_key_val_pairs<T: IntoIterator<Item=U>, U: Into<String>>(args: T) -> Result<HashMap<String, String>, ConfigError> {
    fn left_and_right_hand_side(a: &str) -> Result<(&str, &str), ConfigError> {
        let lr: Vec<&str> = a.split('=').collect();
        if lr.len() != 2 {
            Err(ConfigError::new(a, "is a badly formed argument"))
        } else {
            Ok((lr[0], lr[1]))
        }
    }
    let mut result = HashMap::new();
    for arg in args {
        let str_arg: String = arg.into();
        let (key, value) = left_and_right_hand_side(&str_arg)?;
        if result.contains_key(&str_arg) {
            return Err(ConfigError::new(key, "duplicate parameter"));
        }
        result.insert(key.to_string(), value.to_string());
    }
    Ok(result)
}




// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn make_example_form() -> Form {
        Form::new()
            .item("num_zones" , 5000   , "Number of grid cells to use")
            .item("tfinal"    , 0.2    , "Time at which to stop the simulation")
            .item("rk_order"  , 2      , "Runge-Kutta time integration order")
            .item("quiet"     , false  , "Suppress the iteration message")
    }

    #[test]
    fn can_merge_in_command_line_args() {
        let args = to_string_map_from_key_val_pairs(std::env::args().skip(1)).unwrap();
        let form = make_example_form()
            .merge_string_map(args)
            .unwrap();
        assert!(form.get("num_zones").as_int() == 5000);
    }

    #[test]
    fn can_merge_vector_of_args() {
        let args = to_string_map_from_key_val_pairs(vec!["tfinal=0.4", "rk_order=1", "quiet=true"]).unwrap();
        let form = make_example_form()
            .merge_string_map(args)
            .unwrap();
        assert!(form.get("num_zones").as_int() == 5000);
        assert!(form.get("tfinal").as_float() == 0.4);
        assert!(form.get("rk_order").as_int() == 1);
        assert!(form.get("quiet").as_bool() == true);
    }

    #[test]
    fn can_merge_value_map() {
        let args: HashMap<String, Value> = vec![
            ("num_zones".to_string(), Value::from(2000)),
            ("quiet".to_string(), Value::from(true))]
        .into_iter()
        .collect();

        let form = make_example_form()
            .merge_value_map(&args)
            .unwrap();

        assert!(form.get("num_zones").as_int() == 2000);
        assert!(form.get("tfinal").as_float() == 0.2);
        assert!(form.get("rk_order").as_int() == 2);
        assert!(form.get("quiet").as_bool() == true);
    }

    #[test]
    fn merge_value_map_fails_with_kind_mismatch() {
        let args: HashMap<String, Value> = vec![
            ("num_zones".to_string(), Value::from(3.14)),
            ("quiet".to_string(), Value::from(true))]
        .into_iter()
        .collect();

        assert!(make_example_form().merge_value_map(&args).is_err());
    }
}
