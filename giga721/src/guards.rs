
use crate::token::State;
use ic_cdk::{caller};

#[inline(always)]
pub fn owner_guard() -> Result<(), String> {
    let owner = State::get().borrow().owner.ok_or_else(|| String::from("Owner not set"))?;

    if caller() == owner {
        Ok(())
    } else {
        Err(String::from("The caller is not the owner of contract"))
    }
}

#[inline(always)]
#[allow(dead_code)]
pub fn not_paused() -> Result<(), String> {
    if State::get().borrow().is_paused { return Err(String::from("Canister is paused, try another time")); }

    Ok(())
}