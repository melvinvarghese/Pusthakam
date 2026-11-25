//! One permission-checked implementation for desktop IPC and future browser dispatch.
//! Defining a shared handler does not expose it through HTTP.

#[macro_export]
macro_rules! shared_command {
    (@implementation $(#[$attribute:meta])* $name:ident ($state:ident $(, $arg:ident : $argty:ty)*) -> $result:ty $body:block) => {
        pub mod $name {
            use super::*;

            $(#[$attribute])*
            pub fn execute($state: &$crate::state::AppState, $($arg: $argty),*) -> $result $body

            pub fn dispatch(
                state: &$crate::state::AppState,
                arguments: serde_json::Value,
            ) -> $crate::error::AppResult<serde_json::Value> {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase", deny_unknown_fields)]
                struct Arguments { $($arg: $argty),* }
                let arguments: Arguments = serde_json::from_value(arguments).map_err(|_| {
                    $crate::error::AppError::Validation("Invalid command arguments.".into())
                })?;
                let _ = &arguments;
                let result = execute(state, $(arguments.$arg),*)?;
                serde_json::to_value(result).map_err(|_| {
                    $crate::error::AppError::Internal("Could not encode command result.".into())
                })
            }
        }
    };
    ($(#[$attribute:meta])* pub fn $name:ident(
        $state:ident : State<$lifetime:lifetime, AppState> $(, $arg:ident : $argty:ty)* $(,)?
    ) -> $result:ty $body:block) => {
        $crate::shared_command!(@implementation $(#[$attribute])* $name ($state $(, $arg: $argty)*) -> $result $body);
        $(#[$attribute])*
        #[tauri::command]
        pub fn $name($state: State<$lifetime, AppState>, $($arg: $argty),*) -> $result {
            $name::execute($state.inner(), $($arg),*)
        }
    };
    ($(#[$attribute:meta])* pub async fn $name:ident(
        $state:ident : State<$lifetime:lifetime, AppState> $(, $arg:ident : $argty:ty)* $(,)?
    ) -> $result:ty $body:block) => {
        $crate::shared_command!(@implementation $(#[$attribute])* $name ($state $(, $arg: $argty)*) -> $result $body);
        $(#[$attribute])*
        #[tauri::command]
        pub async fn $name($state: State<$lifetime, AppState>, $($arg: $argty),*) -> $result {
            $name::execute($state.inner(), $($arg),*)
        }
    };
}
