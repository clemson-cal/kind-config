use std::collections::HashMap;
use std::fmt;




/**
 * Enum (variant) whose kind is either bool, int, float, or string. These are
 * the types of values allowed in a Form.
 */
#[derive(Clone)]
pub enum Value {
    B(bool),
    I(i64),
    F(f64),
    S(String),
}

impl Value {

    /**
     * Determine whether this value and another are of the same kind.
     */
    pub fn same_kind_as(&self, other: &Value) -> bool {
       match (&self, &other) {
           (Value::B(_), Value::B(_)) => true,
           (Value::I(_), Value::I(_)) => true,
           (Value::F(_), Value::F(_)) => true,
           (Value::S(_), Value::S(_)) => true,
           _ => false,
       }
    }

    pub fn same_as(&self, other: &Value) -> bool {
       match (&self, &other) {
           (Value::B(a), Value::B(b)) => a == b,
           (Value::I(a), Value::I(b)) => a == b,
           (Value::F(a), Value::F(b)) => a == b,
           (Value::S(a), Value::S(b)) => a == b,
           _ => false,
       }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Value::S(s) => &s,
            _ => panic!(),
        }
    }
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

impl From<bool> for Value { fn from(a: bool) -> Self { Value::B(a) } }
impl From<i64>  for Value { fn from(a: i64)  -> Self { Value::I(a) } }
impl From<f64>  for Value { fn from(a: f64)  -> Self { Value::F(a) } }
impl From<&str> for Value { fn from(a: &str) -> Self { Value::S(a.into()) } }

impl<'a> From<&'a Value> for bool   { fn from(a: &'a Value) -> bool   { match a { Value::B(x) => x.clone(), _ => panic!() } } }
impl<'a> From<&'a Value> for i64    { fn from(a: &'a Value) -> i64    { match a { Value::I(x) => x.clone(), _ => panic!() } } }
impl<'a> From<&'a Value> for f64    { fn from(a: &'a Value) -> f64    { match a { Value::F(x) => x.clone(), _ => panic!() } } }
impl<'a> From<&'a Value> for String { fn from(a: &'a Value) -> String { match a { Value::S(x) => x.clone(), _ => panic!() } } }




// ============================================================================
#[derive(Debug)]
pub struct ConfigError {
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




/**
 * A value and an about string. This is the value type of the HashMap used in a
 * Form.
 */
#[derive(Clone)]
pub struct Parameter {
    pub value: Value,
    pub about: String,
    pub frozen: bool,
}




/**
 * A configuration data structure that is kind-checked at runtime. Items are
 * declared using the `item` member function, after which their value can be
 * updated but their kind (bool, int, float, string) cannot change.
 */
pub struct Form {
    parameter_map: HashMap::<String, Parameter>
}




// ============================================================================
impl Form {

    /**
     * Create a blank form
     */
    pub fn new() -> Form {
        Form{parameter_map: HashMap::new()}
    }

    /**
     * Declare a new config item. Any item already declared with that name is
     * replaced.
     *
     * # Arguments
     *
     * * `key`     - The name of the config item
     * * `default` - The default value
     * * `about`   - A description of the item for use in user reporting
     */
    pub fn item<T: Into<Value>>(mut self, key: &str, default: T, about: &str) -> Self {
        self.parameter_map.insert(key.into(), Parameter{value: default.into(), about: about.into(), frozen: false});
        return self
    }

    /**
     * Merge in the contents of a string-value map, and freeze any of those items
     * which are named in the given vector of keys to be frozen.
     *
     * # Arguments
     *
     * * `items` - A map of values to update the map with
     * * `to_freeze` - A vector of keys to freeze, if the key is in `items`
     */
    pub fn merge_value_map_freezing(self, items: &HashMap<String, Value>, to_freeze: &Vec<&str>) -> Result<Self, ConfigError> {
        let mut result = self.merge_value_map(items)?;
        for key in to_freeze {
            if items.contains_key(*key) {
                result.parameter_map.get_mut(*key).unwrap().frozen = true;
            }
        }
        Ok(result)
    }

    /**
     * Merge in the contents of a string-value map. The result is an error if
     * any of the new keys have not already been declared in the form, or if
     * they were declared as a different type.
     *
     * # Arguments
     *
     * * `items` - A map of values to update the map with
     */
    pub fn merge_value_map(mut self, items: &HashMap<String, Value>) -> Result<Self, ConfigError> {
        for (key, new_value) in items {
            if let Some(item) = self.parameter_map.get_mut(key) {
                if ! item.value.same_kind_as(new_value) {
                    return Err(ConfigError::new(key, "has the wrong type"));
                } else if item.frozen && ! item.value.same_as(new_value) {
                    return Err(ConfigError::new(key, "cannot be modified"));
                } else {
                    item.value = new_value.clone();
                }
            } else {
                return Err(ConfigError::new(key, "is not a valid key"));
            }
        }
        Ok(self)
    }

    /**
     * Merge in the contents of a string-string map. The result is an error if
     * any of the new keys have not already been declared in the form, or if
     * any of the value strings do not parse to the declared type.
     *
     * # Arguments
     *
     * * `dict` - A map of string to update the map with
     */
    pub fn merge_string_map(self, dict: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let items = self.string_map_to_value_map(dict)?;
        self.merge_value_map(&items)
    }

    /**
     * Merge in a sequence of "key=value" pairs. The result is an error if any of
     * the new keys have not already been declared in the form, or if any of the
     * value strings do not parse to the declared type.
     *
     * # Arguments
     *
     * * `args` - Iterator of string to update the map with
     *
     * # Example
     * ```
     * # let base = kind_config::Form::new();
     * let form = base.merge_string_args(std::env::args().skip(1)).unwrap();
     * ```
     */
    pub fn merge_string_args<T: IntoIterator<Item=U>, U: Into<String>>(self, args: T) -> Result<Self, ConfigError> {
        to_string_map_from_key_val_pairs(args).map(|res| self.merge_string_map(&res))?
    }

    pub fn merge_string_args_allowing_duplicates<T: IntoIterator<Item=U>, U: Into<String>>(self, args: T) -> Result<Self, ConfigError> {
        to_string_map_from_key_val_pairs_allowing_duplicates(args).map(|res| self.merge_string_map(&res))?
    }

    /**
     * Freeze a parameter with the given name, if it exists, or otherwise panic.
     */
    pub fn freeze(mut self, key: &str) -> Self
    {
        self.parameter_map.get_mut(key).unwrap().frozen = true;
        return self;
    }

    /**
     * Return a hash map of the (key, value) items, stripping out the about
     * messages. If the HDF5 feature is enabled, the result can be written
     * directly to an HDF5 group via io::write_to_hdf5.
     */
    pub fn value_map(&self) -> HashMap::<String, Value> {
        self.parameter_map.iter().map(|(key, parameter)| (key.clone(), parameter.value.clone())).collect()
    }

    /**
     * Return the number of items.
     */
    pub fn len(&self) -> usize {
        self.parameter_map.len()
    }

    /**
     * Return a vector of the keys in this map, sorted alphabetically.
     */
    pub fn sorted_keys(&self) -> Vec<String>
    {
        let mut result: Vec<String> = self.parameter_map.keys().map(|x| x.to_string()).collect();
        result.sort();
        result
    }

    /**
     * Get an item from the form. Panics if the item was not declared.
     * 
     * # Arguments
     * 
     * * `key` - The key to get
     * 
     * # Example
     * ```
     * # let form = kind_config::Form::new().item("counter", 0, "");
     * let x: i64 = form.get("counter").into(); // fails unless "counter" is declared has kind i64
     * ```
     */
    pub fn get(&self, key: &str) -> &Value {
        &self.parameter_map.get(key.into()).unwrap().value
    }

    pub fn about(&self, key: &str) -> &str {
        &self.parameter_map.get(key.into()).unwrap().about
    }

    pub fn is_frozen(&self, key: &str) -> bool {
        self.parameter_map.get(key.into()).unwrap().frozen
    }

    fn string_map_to_value_map(&self, dict: &HashMap<String, String>) -> Result<HashMap<String, Value>, ConfigError> {
        use Value::*;

        let mut result = HashMap::new();

        for (k, v) in dict {
            let parameter = self.parameter_map.get(k).ok_or(ConfigError::new(&k, "is not a valid key"))?;
            let value = match parameter.value {
                B(_) => v.parse().map(|x| B(x)).map_err(|_| ConfigError::new(k, "is a badly formed bool")),
                I(_) => v.parse().map(|x| I(x)).map_err(|_| ConfigError::new(k, "is a badly formed int")),
                F(_) => v.parse().map(|x| F(x)).map_err(|_| ConfigError::new(k, "is a badly formed float")),
                S(_) => v.parse().map(|x| S(x)).map_err(|_| ConfigError::new(k, "is a badly formed string")),
            }?;
            result.insert(k.to_string(), value);
        }
        Ok(result)
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
fn to_string_map_from_key_val_pairs_general<T: IntoIterator<Item=U>, U: Into<String>>(args: T, allow_duplicates: bool) -> Result<HashMap<String, String>, ConfigError> {
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
        if !allow_duplicates && result.contains_key(key) {
            return Err(ConfigError::new(key, "duplicate parameter"));
        }
        result.insert(key.to_string(), value.to_string());
    }
    Ok(result)
}

pub fn to_string_map_from_key_val_pairs<T: IntoIterator<Item=U>, U: Into<String>>(args: T) -> Result<HashMap<String, String>, ConfigError> {
    to_string_map_from_key_val_pairs_general(args, false)
}

pub fn to_string_map_from_key_val_pairs_allowing_duplicates<T: IntoIterator<Item=U>, U: Into<String>>(args: T) -> Result<HashMap<String, String>, ConfigError> {
    to_string_map_from_key_val_pairs_general(args, true)
}




// ============================================================================
#[cfg(feature="hdf5")]
pub mod io {
    use hdf5;
    use super::*;

    pub fn write_to_hdf5(group: &hdf5::Group, value_map: &HashMap::<String, Value>) -> Result<(), hdf5::Error> {
        use hdf5::types::VarLenAscii;

        for (key, value) in value_map {
            match &value {
                Value::B(x) => group.new_dataset::<bool>().create(key, ())?.write_scalar(x),
                Value::I(x) => group.new_dataset::<i64>().create(key, ())?.write_scalar(x),
                Value::F(x) => group.new_dataset::<f64>().create(key, ())?.write_scalar(x),
                Value::S(x) => group.new_dataset::<VarLenAscii>().create(key, ())?.write_scalar(&VarLenAscii::from_ascii(&x).unwrap()),
            }?;
        }
        Ok(())
    }

    pub fn read_from_hdf5(group: &hdf5::Group) -> Result<HashMap::<String, Value>, hdf5::Error> {
        use hdf5::types::VarLenAscii;
        let mut values = HashMap::<String, Value>::new();

        for key in group.member_names()? {
            let dtype = group.dataset(&key)?.dtype()?;
            let value =
            if dtype.is::<bool>() {
                group.dataset(&key)?.read_scalar::<bool>().map(|x| Value::from(x))
            } else if dtype.is::<i64>() {
                group.dataset(&key)?.read_scalar::<i64>().map(|x| Value::from(x))
            } else if dtype.is::<f64>() {
                group.dataset(&key)?.read_scalar::<f64>().map(|x| Value::from(x))
            } else {
                group.dataset(&key)?.read_scalar::<VarLenAscii>().map(|x| Value::from(x.as_str()))
            }?;
            values.insert(key.to_string(), value);
        }
        Ok(values)
    }
}




// ============================================================================
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::Value;
    use crate::Form;
    use crate::to_string_map_from_key_val_pairs;

    fn make_example_form() -> Form {
        Form::new()
            .item("num_zones" , 5000   , "Number of grid cells to use")
            .item("tfinal"    , 0.2    , "Time at which to stop the simulation")
            .item("rk_order"  , 2      , "Runge-Kutta time integration order")
            .item("quiet"     , false  , "Suppress the iteration message")
            .item("outdir"    , "data" , "Directory where output data is written")
    }

    #[test]
    fn can_freeze_parameter() {
        let form = make_example_form().freeze("num_zones");
        assert!(  form.is_frozen("num_zones"));
        assert!(! form.is_frozen("outdir"));
    }

    #[test]
    fn can_merge_in_command_line_args() {
        let form = make_example_form()
            .merge_string_args(std::env::args().skip(1))
            .unwrap();
        assert!(i64::from(form.get("num_zones")) == 5000);
    }

    #[test]
    fn can_merge_vector_of_args() {
        let args = to_string_map_from_key_val_pairs(vec!["tfinal=0.4", "rk_order=1", "quiet=true"]).unwrap();
        let form = make_example_form()
            .merge_string_map(&args)
            .unwrap();
        assert!(i64::from(form.get("num_zones")) == 5000);
        assert!(f64::from(form.get("tfinal")) == 0.4);
        assert!(i64::from(form.get("rk_order")) == 1);
        assert!(bool::from(form.get("quiet")) == true);
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

        assert!(i64::from(form.get("num_zones")) == 2000);
        assert!(f64::from(form.get("tfinal")) == 0.2);
        assert!(i64::from(form.get("rk_order")) == 2);
        assert!(bool::from(form.get("quiet")) == true);
    }

    #[test]
    fn can_merge_freeze_value_map() {
        let args: HashMap<String, Value> = vec![
            ("num_zones".to_string(), Value::from(2000)),
            ("quiet".to_string(), Value::from(true))]
        .into_iter()
        .collect();

        let form = make_example_form()
            .merge_value_map_freezing(&args, &vec!["num_zones", "rk_order"])
            .unwrap();

        assert!(  form.is_frozen("num_zones"));
        assert!(! form.is_frozen("rk_order"));
    }

    #[test]
    #[should_panic]
    fn to_string_map_fails_with_duplicate_parameter() {
        to_string_map_from_key_val_pairs(vec!["a=1".to_string(), "a=2".to_string()]).unwrap();
    }

    #[test]
    #[should_panic]
    fn to_string_map_fails_with_badly_formed_parameter() {
        to_string_map_from_key_val_pairs(vec!["a 2".to_string()]).unwrap();
    }

    #[test]
    #[should_panic]
    fn merge_value_map_fails_with_kind_mismatch() {
        let args: HashMap<String, Value> = vec![
            ("num_zones".to_string(), Value::from(3.14)),
            ("quiet".to_string(), Value::from(true))]
        .into_iter()
        .collect();
        make_example_form().merge_value_map(&args).unwrap();
    }

    #[cfg(feature="hdf5")]
    #[cfg(test)]
    mod io_tests {
        use super::*;

        #[test]
        fn can_write_to_hdf5() {
            let file = hdf5::File::create("test1.h5").unwrap();
            let form = make_example_form();
            io::write_to_hdf5(&form.value_map(), &file).unwrap();
        }

        #[test]
        fn can_read_from_hdf5() {
            io::write_to_hdf5(&make_example_form().value_map(), &hdf5::File::create("test2.h5").unwrap()).unwrap();
            let file = hdf5::File::open("test2.h5").unwrap();
            let value_map = io::read_from_hdf5(&file).unwrap();
            let form = make_example_form().merge_value_map(&value_map).unwrap();
            assert_eq!(form.len(), 5);
            assert_eq!(form.get("num_zones").as_int(), 5000);
        }
    }
}
