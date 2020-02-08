/**
 * utils/macros.rs
 *
 * Various macro utils, ranging from SIR-related uses to misc bookkeeping stuff.
 * Everything is bundle in a single file (for now) because rust macros don't have namespaces
 *
 **/
#[allow(unused_macros)]
macro_rules! optionally_push {
    ($vec: ident, $opt: expr) => {
        match $opt {
            Some(v) => $vec.push(v),
            None => {},
        }
    }
}

macro_rules! push_all_clone {
    ($vec: expr, $($othervec: expr),*) => {
        $(
            for elem in $othervec {
                $vec.push(elem.clone());
            }
        )*
    }
}

//TODO: add constructor
//this and use default location for now
#[allow(unused_macros)]
macro_rules! make_vm_struct {
    ($struct_name: ident, $name: ty, $os: ty, $hwconfig: ty, $location: ty, $auth_type: ty, $custom_script: ty) => {
        #[derive(Debug)]
        pub struct $struct_name {
            pub name: $name,
            pub os: $os,
            pub hwconfig: $hwconfig,
            pub auth_type: $auth_type,
            pub custom_script: $custom_script
        }
    };
}

macro_rules! yaml_tree_from_file {
    ($file: expr) => { 
            {
            let text = yaml_to_str($file);
            &YamlLoader::load_from_str(&text).unwrap_or_else(|_| panic!("Could not parse {} into YAML", $file))[0]
        }
    }
}

macro_rules! make_getter {
    ($($field: ident ~ $tpe: ty), *) => {
        $(fn $field(&self) -> $tpe {
            self.$field
        })*
    }
}

macro_rules! missing_getter {
    ($($field: ident ~  $tpe: ty), *) => {
        $(fn $field(&self) -> $tpe {
            unimplemented!()
        })*
    }
}

macro_rules! within_bounds_incl {
    ($low: expr, $num: expr, $high: expr) => {

        $num >= $low && $num <= $high
    }
}

// small convenience macro because Rust does not allow to use 'continue' inside a closure
macro_rules! unwrap_or_continue {
    ($opt: expr) => {
        {
            if $opt.is_none() {
                continue;
            }
            $opt.unwrap()
        }
    }
}
