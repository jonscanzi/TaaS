// yamlsir/default.rs
//
// Simple definitions of getter function to access default values for the
// YAML system description file. These functions are required for
// serde's system to generate default values
//
// Note: an appropriate default value must be present in the global_config mod
use crate::utils::global_config::DEFAULT_VALUES;

// TODO: make this macro allow arbitrary number of
// tuples to avoid calling it a lot of times
macro_rules! make_default {
    ($field: ident, $rettype: tt) => {
        #[inline]
        #[allow(dead_code)]
        pub fn $field() -> $rettype {
            DEFAULT_VALUES.$field.clone()
        }
    }
}

make_default!(cpu_freq_mhz, usize);
make_default!(cpu_cores, usize);
make_default!(ram_gb, usize);

make_default!(capacity_gb, usize);
make_default!(r#type, String);
make_default!(grade, u8);

make_default!(os_common, String);

make_default!(location, String);
make_default!(remote_access, bool);