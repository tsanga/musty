pub(crate) mod string {
    /// convert string to table case. i.e "MyUser" -> "my_user"
    pub trait ToTableCase {
        fn to_table_case(self) -> String;
    }

    impl<S> ToTableCase for S
    where
        S: Into<String>,
    {
        fn to_table_case(self) -> String {
            let str: String = self.into();
            let mut table_case = String::new();
            let mut prev_char = '_';
            for c in str.chars() {
                if c.is_uppercase() && prev_char != '_' {
                    table_case.push('_');
                }
                table_case.push(c.to_ascii_lowercase());
                prev_char = c;
            }
            table_case
        }
    }

    /// convert string to plural form. i.e: "user" -> "users"
    pub trait ToPlural {
        fn to_plural(&self) -> String;
    }

    impl ToPlural for String {
        // if string doesn't end with an 's', add one
        fn to_plural(&self) -> String {
            if self.ends_with('s') {
                self.clone()
            } else {
                format!("{}s", self)
            }
        }
    }
}
