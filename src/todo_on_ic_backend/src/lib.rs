use candid::CandidType;
use ic_cdk::{
    api::{caller, time},
    query, update,
};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use serde::Deserialize;
use std::{cell::RefCell, collections::HashMap, thread_local};
use uuid::{timestamp::context::Context, Timestamp, Uuid};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Debug, CandidType, Deserialize, Eq, PartialEq)]
enum IcResult {
    Ok(String),
    Err(String),
}

thread_local! {
    // retain TODO list after canister updates
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static TODO_LIST: RefCell<StableBTreeMap<String, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[update]
fn create_todo(note: String) -> String {
    // ideally uuid should always give unique string
    let context = Context::new(time() as u16);
    let t64 = time();
    let ts = Timestamp::from_unix(&context, t64, t64 as u32);
    let node_id = caller().to_text().into_bytes();
    let node_id: &[u8; 6] = node_id.as_slice()[..6].try_into().unwrap();
    let mut id = Uuid::new_v1(ts, node_id).to_string();
    TODO_LIST.with(|todo_list| {
        // should ideally limit number of retries or give permanent error
        while todo_list.borrow().contains_key(&id) {
            let t64 = time();
            let ts = Timestamp::from_unix(&context, t64, t64 as u32);
            id = Uuid::new_v1(ts, node_id).to_string();
        }
        todo_list.borrow_mut().insert(id.to_owned(), note);
    });
    id
}

#[query]
fn get_todo_by_id(task_id: String) -> IcResult {
    TODO_LIST.with(|todo_list| match todo_list.borrow().get(&task_id) {
        Some(todo_text) => IcResult::Ok(todo_text.to_owned()),
        None => IcResult::Err(format!("No task found with id: {}", task_id)),
    })
}

#[query]
fn get_todos_by_page(page_number: u32, per_page: u32) -> HashMap<String, String> {
    // page_number starts with 1
    let page_number = if page_number == 0 {
        1
    } else {
        page_number as usize
    };
    // atleast 1 per page will be displayed and maximum 10 allowed
    let per_page = if per_page > 10 {
        10
    } else if per_page == 0 {
        1
    } else {
        per_page as usize
    };
    let mut todos: HashMap<String, String> = HashMap::with_capacity(per_page);
    TODO_LIST.with(|todo_list| {
        let _ = todo_list
            .borrow()
            .iter()
            .skip((page_number - 1) * per_page)
            .take(per_page)
            .map(|(k, v)| todos.insert(k.to_owned(), v.to_owned()))
            .collect::<Vec<_>>();
    });
    todos
}

#[update]
fn update_todo_by_id(task_id: String, note: String) -> IcResult {
    TODO_LIST.with(|todo_list| {
        if !todo_list.borrow().contains_key(&task_id) {
            return IcResult::Err(format!("No task found with id: {}", task_id));
        }
        todo_list.borrow_mut().insert(task_id.to_owned(), note);
        IcResult::Ok(format!("Successfully updated task id: {}", task_id))
    })
}

#[update]
fn delete_todo_by_id(task_id: String) -> IcResult {
    TODO_LIST.with(|todo_list| match todo_list.borrow_mut().remove(&task_id) {
        Some(_) => IcResult::Ok(format!("Successfully deleted task id: {}", task_id)),
        None => IcResult::Err(format!("No task found with id: {}", task_id)),
    })
}

// Enable Candid export
ic_cdk::export_candid!();
