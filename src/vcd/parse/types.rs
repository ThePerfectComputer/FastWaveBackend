// Copyright (C) 2022 Yehowshua Immanuel
// This program is distributed under both the GPLV3 license
// and the YEHOWSHUA license, both of which can be found at
// the root of the folder containing the sources for this program.
#[derive(Debug)]
pub(super) struct ParseResult<'a> {
    pub(super) matched: &'a str,
    pub(super) residual: &'a str,
}

impl<'a> ParseResult<'a> {
    pub(super) fn assert_match(&self) -> Result<&str, String> {
        if self.matched.is_empty() {
            Err("no match".to_string())
        } else {
            Ok(self.matched)
        }
    }

    pub(super) fn assert_residual(&self) -> Result<&str, String> {
        if self.residual.is_empty() {
            Err("no residual".to_string())
        } else {
            Ok(self.residual)
        }
    }
}
