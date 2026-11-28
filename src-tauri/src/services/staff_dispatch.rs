//! Explicit typed staff dispatch; session/origin/CSRF policy belongs to the HTTP gateway.
use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use serde::Deserialize;
use serde_json::{json, Value};

pub fn dispatch(
    state: &AppState,
    command: &str,
    args: Value,
    local_setup: bool,
) -> AppResult<Value> {
    match command {
        "setup_status" => return crate::commands::auth::setup_status::dispatch(state, args),
        "current_session" => return crate::commands::auth::current_session::dispatch(state, args),
        "login" => return crate::commands::auth::login::dispatch(state, args),
        "logout" => return crate::commands::auth::logout::dispatch(state, args),
        "staff_recover" => return crate::commands::staff::staff_recover::dispatch(state, args),
        "complete_setup" if local_setup => {
            return crate::commands::auth::complete_setup::dispatch(state, args)
        }
        "complete_setup" => {
            return Err(AppError::Permission(
                "Set up the library from the host computer before enabling remote access.".into(),
            ))
        }
        "app_info" => {
            return Ok(
                json!({"version":env!("CARGO_PKG_VERSION"),"schemaVersion":crate::db::migrations::latest_version(),"dataDir":"","databasePath":""}),
            )
        }
        _ => {}
    }
    state.current()?;
    match command {
        "catalogue_save_export" | "reports_save_export" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Save {
                file_name: String,
                data_base64: String,
            }
            let input: Save = serde_json::from_value(args)
                .map_err(|_| AppError::Validation("Invalid export arguments.".into()))?;
            if command == "catalogue_save_export" {
                state.require("catalogue.read")?;
                crate::services::export_save::prepare(&input.file_name, &input.data_base64)?;
            } else {
                state.require("reports.read")?;
                crate::services::export_save::prepare_report(&input.file_name, &input.data_base64)?;
            }
            Ok(json!({"native":false,"path":null}))
        }
        "backup_copy_to_folder" => Err(AppError::Validation(
            "Download the backup to this device instead of supplying a server folder.".into(),
        )),
        "backup_set_daily_folder" | "backup_pick_folder" => Err(AppError::Validation(
            "Choose the daily backup folder on the computer that runs Pusthakam.".into(),
        )),
        "launch_at_startup_get" | "launch_at_startup_set" => Err(AppError::Validation(
            "Change starting with Windows on the computer that runs Pusthakam.".into(),
        )),
        "dashboard_overview" => crate::commands::app::dashboard_overview::dispatch(state, args),
        "get_setting" => crate::commands::app::get_setting::dispatch(state, args),
        "set_setting" => crate::commands::app::set_setting::dispatch(state, args),
        "config_overview" => crate::commands::config::config_overview::dispatch(state, args),
        "config_calendar" => crate::commands::config::config_calendar::dispatch(state, args),
        "config_save_closed_weekdays" => {
            crate::commands::config::config_save_closed_weekdays::dispatch(state, args)
        }
        "config_save_holiday" => {
            crate::commands::config::config_save_holiday::dispatch(state, args)
        }
        "config_remove_holiday" => {
            crate::commands::config::config_remove_holiday::dispatch(state, args)
        }
        "config_update_library" => {
            crate::commands::config::config_update_library::dispatch(state, args)
        }
        "config_save_named" => crate::commands::config::config_save_named::dispatch(state, args),
        "config_save_member_category" => {
            crate::commands::config::config_save_member_category::dispatch(state, args)
        }
        "config_save_shelf" => crate::commands::config::config_save_shelf::dispatch(state, args),
        "config_save_policy" => crate::commands::config::config_save_policy::dispatch(state, args),
        "config_delete_policy" => {
            crate::commands::config::config_delete_policy::dispatch(state, args)
        }
        "config_set_archived" => {
            crate::commands::config::config_set_archived::dispatch(state, args)
        }
        "catalogue_search" => crate::commands::catalogue::catalogue_search::dispatch(state, args),
        "catalogue_get_record" => {
            crate::commands::catalogue::catalogue_get_record::dispatch(state, args)
        }
        "catalogue_save_record" => {
            crate::commands::catalogue::catalogue_save_record::dispatch(state, args)
        }
        "catalogue_find_duplicates" => {
            crate::commands::catalogue::catalogue_find_duplicates::dispatch(state, args)
        }
        "catalogue_archive_record" => {
            crate::commands::catalogue::catalogue_archive_record::dispatch(state, args)
        }
        "catalogue_archive_records" => {
            crate::commands::catalogue::catalogue_archive_records::dispatch(state, args)
        }
        "catalogue_get_cover" => {
            crate::commands::catalogue::catalogue_get_cover::dispatch(state, args)
        }
        "catalogue_set_cover" => {
            crate::commands::catalogue::catalogue_set_cover::dispatch(state, args)
        }
        "catalogue_add_copies" => {
            crate::commands::catalogue::catalogue_add_copies::dispatch(state, args)
        }
        "catalogue_update_copy" => {
            crate::commands::catalogue::catalogue_update_copy::dispatch(state, args)
        }
        "catalogue_set_copy_status" => {
            crate::commands::catalogue::catalogue_set_copy_status::dispatch(state, args)
        }
        "catalogue_list_copies" => {
            crate::commands::catalogue::catalogue_list_copies::dispatch(state, args)
        }
        "catalogue_export_copies" => {
            crate::commands::catalogue::catalogue_export_copies::dispatch(state, args)
        }
        "catalogue_export_records" => {
            crate::commands::catalogue::catalogue_export_records::dispatch(state, args)
        }
        "accession_format_get" => {
            crate::commands::catalogue::accession_format_get::dispatch(state, args)
        }
        "accession_format_set" => {
            crate::commands::catalogue::accession_format_set::dispatch(state, args)
        }
        "import_preview" => crate::commands::import::import_preview::dispatch(state, args),
        "import_commit" => crate::commands::import::import_commit::dispatch(state, args),
        "import_cancel" => crate::commands::import::import_cancel::dispatch(state, args),
        "members_list" => crate::commands::members::members_list::dispatch(state, args),
        "members_list_loans" => crate::commands::members::members_list_loans::dispatch(state, args),
        "members_fine_account" => {
            crate::commands::members::members_fine_account::dispatch(state, args)
        }
        "members_fine_history" => {
            crate::commands::members::members_fine_history::dispatch(state, args)
        }
        "members_save" => crate::commands::members::members_save::dispatch(state, args),
        "members_archive" => crate::commands::members::members_archive::dispatch(state, args),
        "members_preview_koha" => {
            crate::commands::members::members_preview_koha::dispatch(state, args)
        }
        "members_import_koha" => {
            crate::commands::members::members_import_koha::dispatch(state, args)
        }
        "members_get_photo" => crate::commands::members::members_get_photo::dispatch(state, args),
        "members_set_photo" => crate::commands::members::members_set_photo::dispatch(state, args),
        "members_set_opac_credentials" => crate::commands::members::members_set_opac_credentials::dispatch(state, args),
        "members_summary" => crate::commands::members::members_summary::dispatch(state, args),
        "members_renew" => crate::commands::members::members_renew::dispatch(state, args),
        "members_holds" => crate::commands::members::members_holds::dispatch(state, args),
        "members_activity" => crate::commands::members::members_activity::dispatch(state, args),
        "circulation_preview_import" => {
            crate::commands::circulation::circulation_preview_import::dispatch(state, args)
        }
        "circulation_preview_history" => {
            crate::commands::circulation::circulation_preview_history::dispatch(state, args)
        }
        "circulation_history_imports" => {
            crate::commands::circulation::circulation_history_imports::dispatch(state, args)
        }
        "circulation_commit_history" => {
            crate::commands::circulation::circulation_commit_history::dispatch(state, args)
        }
        "circulation_preview_renewal_import" => {
            crate::commands::circulation::circulation_preview_renewal_import::dispatch(state, args)
        }
        "circulation_commit_renewal_import" => {
            crate::commands::circulation::circulation_commit_renewal_import::dispatch(state, args)
        }
        "circulation_commit_import" => {
            crate::commands::circulation::circulation_commit_import::dispatch(state, args)
        }
        "circulation_prepare_issue" => {
            crate::commands::circulation::circulation_prepare_issue::dispatch(state, args)
        }
        "circulation_prepare_return" => {
            crate::commands::circulation::circulation_prepare_return::dispatch(state, args)
        }
        "circulation_issue" => {
            crate::commands::circulation::circulation_issue::dispatch(state, args)
        }
        "circulation_return" => {
            crate::commands::circulation::circulation_return::dispatch(state, args)
        }
        "circulation_prepare_renewal" => {
            crate::commands::circulation::circulation_prepare_renewal::dispatch(state, args)
        }
        "circulation_renew" => {
            crate::commands::circulation::circulation_renew::dispatch(state, args)
        }
        "circulation_list_overdue" => {
            crate::commands::circulation::circulation_list_overdue::dispatch(state, args)
        }
        "circulation_list_open_loans" => {
            crate::commands::circulation::circulation_list_open_loans::dispatch(state, args)
        }
        "circulation_list_reservations" => {
            crate::commands::circulation::circulation_list_reservations::dispatch(state, args)
        }
        "circulation_place_reservation" => {
            crate::commands::circulation::circulation_place_reservation::dispatch(state, args)
        }
        "circulation_cancel_reservation" => {
            crate::commands::circulation::circulation_cancel_reservation::dispatch(state, args)
        }
        "circulation_list_fines" => {
            crate::commands::circulation::circulation_list_fines::dispatch(state, args)
        }
        "circulation_settle_fine" => {
            crate::commands::circulation::circulation_settle_fine::dispatch(state, args)
        }
        "circulation_fine_history" => {
            crate::commands::circulation::circulation_fine_history::dispatch(state, args)
        }
        "reports_circulation_activity" => {
            crate::commands::reports::reports_circulation_activity::dispatch(state, args)
        }
        "reports_popular_titles" => {
            crate::commands::reports::reports_popular_titles::dispatch(state, args)
        }
        "reports_stock_by_location" => {
            crate::commands::reports::reports_stock_by_location::dispatch(state, args)
        }
        "reports_fine_summary" => {
            crate::commands::reports::reports_fine_summary::dispatch(state, args)
        }
        "reports_member_summary" => {
            crate::commands::reports::reports_member_summary::dispatch(state, args)
        }
        "reports_catalogue_growth" => {
            crate::commands::reports::reports_catalogue_growth::dispatch(state, args)
        }
        "reports_overdues_list" => {
            crate::commands::reports::reports_overdues_list::dispatch(state, args)
        }
        "reports_accession_list" => {
            crate::commands::reports::reports_accession_list::dispatch(state, args)
        }
        "reports_expenditure_summary" => {
            crate::commands::reports::reports_expenditure_summary::dispatch(state, args)
        }
        "reports_loss_damage_list" => {
            crate::commands::reports::reports_loss_damage_list::dispatch(state, args)
        }
        "reports_export_file" => {
            crate::commands::reports::reports_export_file::dispatch(state, args)
        }
        "backup_list" => crate::commands::backup::backup_list::dispatch(state, args),
        "backup_create" => crate::commands::backup::backup_create::dispatch(state, args),
        "backup_set_daily" => crate::commands::backup::backup_set_daily::dispatch(state, args),
        "backup_verify" => crate::commands::backup::backup_verify::dispatch(state, args),
        "backup_request_restore" => {
            crate::commands::backup::backup_request_restore::dispatch(state, args)
        }
        "backup_cancel_restore" => {
            crate::commands::backup::backup_cancel_restore::dispatch(state, args)
        }
        "audit_search" => crate::commands::audit::audit_search::dispatch(state, args),
        "staff_list" => crate::commands::staff::staff_list::dispatch(state, args),
        "staff_create" => crate::commands::staff::staff_create::dispatch(state, args),
        "staff_update" => crate::commands::staff::staff_update::dispatch(state, args),
        "staff_reset_password" => {
            crate::commands::staff::staff_reset_password::dispatch(state, args)
        }
        "staff_unlock" => crate::commands::staff::staff_unlock::dispatch(state, args),
        "staff_change_password" => {
            crate::commands::staff::staff_change_password::dispatch(state, args)
        }
        "staff_generate_recovery_code" => {
            crate::commands::staff::staff_generate_recovery_code::dispatch(state, args)
        }
        "stock_check_list" => crate::commands::inventory::stock_check_list::dispatch(state, args),
        "stock_check_review" => {
            crate::commands::inventory::stock_check_review::dispatch(state, args)
        }
        "stock_check_start" => crate::commands::inventory::stock_check_start::dispatch(state, args),
        "stock_check_scan" => crate::commands::inventory::stock_check_scan::dispatch(state, args),
        "stock_check_close" => crate::commands::inventory::stock_check_close::dispatch(state, args),
        "stock_transfer_list" => {
            crate::commands::inventory::stock_transfer_list::dispatch(state, args)
        }
        "stock_transfer_send" => {
            crate::commands::inventory::stock_transfer_send::dispatch(state, args)
        }
        "stock_transfer_finish" => {
            crate::commands::inventory::stock_transfer_finish::dispatch(state, args)
        }
        "acquisitions_list_vendors" => {
            crate::commands::acquisitions::acquisitions_list_vendors::dispatch(state, args)
        }
        "acquisitions_save_vendor" => {
            crate::commands::acquisitions::acquisitions_save_vendor::dispatch(state, args)
        }
        "acquisitions_archive_vendor" => {
            crate::commands::acquisitions::acquisitions_archive_vendor::dispatch(state, args)
        }
        "acquisitions_list_budgets" => {
            crate::commands::acquisitions::acquisitions_list_budgets::dispatch(state, args)
        }
        "acquisitions_save_budget" => {
            crate::commands::acquisitions::acquisitions_save_budget::dispatch(state, args)
        }
        "acquisitions_archive_budget" => {
            crate::commands::acquisitions::acquisitions_archive_budget::dispatch(state, args)
        }
        "acquisitions_list_requests" => {
            crate::commands::acquisitions::acquisitions_list_requests::dispatch(state, args)
        }
        "acquisitions_create_request" => {
            crate::commands::acquisitions::acquisitions_create_request::dispatch(state, args)
        }
        "acquisitions_approve_request" => {
            crate::commands::acquisitions::acquisitions_approve_request::dispatch(state, args)
        }
        "acquisitions_reject_request" => {
            crate::commands::acquisitions::acquisitions_reject_request::dispatch(state, args)
        }
        "acquisitions_list_orders" => {
            crate::commands::acquisitions::acquisitions_list_orders::dispatch(state, args)
        }
        "acquisitions_get_order" => {
            crate::commands::acquisitions::acquisitions_get_order::dispatch(state, args)
        }
        "acquisitions_create_order" => {
            crate::commands::acquisitions::acquisitions_create_order::dispatch(state, args)
        }
        "acquisitions_update_order_status" => {
            crate::commands::acquisitions::acquisitions_update_order_status::dispatch(state, args)
        }
        "acquisitions_receive_items" => {
            crate::commands::acquisitions::acquisitions_receive_items::dispatch(state, args)
        }
        "serials_list_subscriptions" => {
            crate::commands::serials::serials_list_subscriptions::dispatch(state, args)
        }
        "serials_get_subscription" => {
            crate::commands::serials::serials_get_subscription::dispatch(state, args)
        }
        "serials_save_subscription" => {
            crate::commands::serials::serials_save_subscription::dispatch(state, args)
        }
        "serials_cancel_subscription" => {
            crate::commands::serials::serials_cancel_subscription::dispatch(state, args)
        }
        "serials_list_issues" => {
            crate::commands::serials::serials_list_issues::dispatch(state, args)
        }
        "serials_list_missing_issues" => {
            crate::commands::serials::serials_list_missing_issues::dispatch(state, args)
        }
        "serials_add_issue" => crate::commands::serials::serials_add_issue::dispatch(state, args),
        "serials_check_in_issue" => {
            crate::commands::serials::serials_check_in_issue::dispatch(state, args)
        }
        "serials_update_issue_status" => {
            crate::commands::serials::serials_update_issue_status::dispatch(state, args)
        }
        "serials_renew_subscription" => {
            crate::commands::serials::serials_renew_subscription::dispatch(state, args)
        }
        "serials_list_renewals" => {
            crate::commands::serials::serials_list_renewals::dispatch(state, args)
        }
        "serials_get_expenditure" => {
            crate::commands::serials::serials_get_expenditure::dispatch(state, args)
        }
        "serials_list_expiring" => {
            crate::commands::serials::serials_list_expiring::dispatch(state, args)
        }
        "classification_list_schemes" => {
            crate::commands::classification::classification_list_schemes::dispatch(state, args)
        }
        "classification_save_scheme" => {
            crate::commands::classification::classification_save_scheme::dispatch(state, args)
        }
        "classification_delete_scheme" => {
            crate::commands::classification::classification_delete_scheme::dispatch(state, args)
        }
        "classification_list_entries" => {
            crate::commands::classification::classification_list_entries::dispatch(state, args)
        }
        "classification_save_entry" => {
            crate::commands::classification::classification_save_entry::dispatch(state, args)
        }
        "classification_delete_entry" => {
            crate::commands::classification::classification_delete_entry::dispatch(state, args)
        }
        "classification_list_authors" => {
            crate::commands::classification::classification_list_authors::dispatch(state, args)
        }
        "classification_save_author" => {
            crate::commands::classification::classification_save_author::dispatch(state, args)
        }
        "classification_merge_authors" => {
            crate::commands::classification::classification_merge_authors::dispatch(state, args)
        }
        "classification_list_publishers" => {
            crate::commands::classification::classification_list_publishers::dispatch(state, args)
        }
        "classification_save_publisher" => {
            crate::commands::classification::classification_save_publisher::dispatch(state, args)
        }
        "classification_merge_publishers" => {
            crate::commands::classification::classification_merge_publishers::dispatch(state, args)
        }
        "classification_list_subjects" => {
            crate::commands::classification::classification_list_subjects::dispatch(state, args)
        }
        "classification_save_subject" => {
            crate::commands::classification::classification_save_subject::dispatch(state, args)
        }
        "classification_merge_subjects" => {
            crate::commands::classification::classification_merge_subjects::dispatch(state, args)
        }
        "browse_categories_list" => {
            crate::commands::classification::browse_categories_list::dispatch(state, args)
        }
        "browse_categories_save" => {
            crate::commands::classification::browse_categories_save::dispatch(state, args)
        }
        "browse_categories_set_archived" => {
            crate::commands::classification::browse_categories_set_archived::dispatch(state, args)
        }
        "classification_list_marc_mappings" => {
            crate::commands::classification::classification_list_marc_mappings::dispatch(
                state, args,
            )
        }
        "classification_save_marc_mapping" => {
            crate::commands::classification::classification_save_marc_mapping::dispatch(state, args)
        }
        "classification_test_marc_mapping" => {
            crate::commands::classification::classification_test_marc_mapping::dispatch(state, args)
        }
        "lan_get_config" => crate::commands::lan::lan_get_config::dispatch(state, args),
        "lan_update_config" => crate::commands::lan::lan_update_config::dispatch(state, args),
        "lan_generate_pairing_code" => {
            crate::commands::lan::lan_generate_pairing_code::dispatch(state, args)
        }
        "lan_list_pairing_codes" => {
            crate::commands::lan::lan_list_pairing_codes::dispatch(state, args)
        }
        "lan_list_paired_clients" => {
            crate::commands::lan::lan_list_paired_clients::dispatch(state, args)
        }
        "lan_revoke_client" => crate::commands::lan::lan_revoke_client::dispatch(state, args),
        "lan_pair_client" => crate::commands::lan::lan_pair_client::dispatch(state, args),
        "lan_login" => crate::commands::lan::lan_login::dispatch(state, args),
        "lan_opac_search" => crate::commands::lan::lan_opac_search::dispatch(state, args),
        "lan_opac_get_record" => crate::commands::lan::lan_opac_get_record::dispatch(state, args),
        _ => Err(AppError::Validation("Unknown staff command.".into())),
    }
}
