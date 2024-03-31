pub enum ScannerError<'a> {
    Err(usize, &'a str),
}
