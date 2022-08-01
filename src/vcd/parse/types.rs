#[derive(Debug)]
pub(super) struct ParseResult<'a> {
    pub(super) matched : &'a str, 
    pub(super) residual : &'a str}

impl<'a> ParseResult<'a> {

    pub(super) fn assert_match(& self) -> Result<&str, String> {
        if self.matched == "" {
            return Err("no match".to_string())
        }
        else {
            return Ok(self.matched)
        }
    }

    pub(super) fn assert_residual(& self) -> Result<&str, String> {
        if self.residual == "" {
            return Err("no residual".to_string())
        }
        else {
            return Ok(self.residual)
        }
    }
}