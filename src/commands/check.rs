use crate::paths::{gubby_installed, warn_no_gubby};

pub fn run() {
    if !gubby_installed() {warn_no_gubby(); return;}
}