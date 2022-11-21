pub enum RequestCmd {
    Ping,
    AddFlowEntry,
    ShowFlowEntry,
}

pub enum ResponseCmd {
    SuccessMessage,
    ErrorMessage,
}
