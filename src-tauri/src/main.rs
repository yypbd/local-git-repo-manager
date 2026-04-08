fn main() {
    let _ = local_repo_manager_lib::app_lock::acquire_single_instance_stub();
    local_repo_manager_lib::run();
}
