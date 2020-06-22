use std::borrow::Borrow;
use std::ops::Neg;
use std::time::SystemTime;

use bigdecimal::BigDecimal;
use diesel::{
	deserialize,
	deserialize::FromSql,
	pg::Pg,
	PgConnection,
	prelude::*,
	serialize,
	serialize::{Output, ToSql},
	sql_types::Varchar,
};

use crate::{Account, AccountType, BankTransactionType, error, PgPool, Result, schema::*};

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
	pub user_id: uuid::Uuid,
	pub account_type: AccountType,
}

pub struct Repo {
	db: PgPool,
}

impl Repo {
	pub fn new(db: PgPool) -> Self {
		Repo { db }
	}
	
	pub fn create_account(&self, new_account: NewAccount) -> Result<Account> {
		let conn = &self.db.get()?;
		diesel::insert_into(accounts::table)
			.values(&new_account)
			.get_result(conn)
			.map_err(Into::into)
	}
	
	pub fn find_accounts(&self, user_id: &uuid::Uuid) -> Result<Vec<Account>> {
		let conn = &self.db.get()?;
		accounts::table
			.filter(accounts::user_id.eq(user_id))
			.select((accounts::all_columns))
			.load::<Account>(conn)
			.map_err(Into::into)
	}
	
	pub fn find_account(&self, account_id: &uuid::Uuid) -> Result<Account> {
		let conn = &self.db.get()?;
		accounts::table
			.filter(accounts::id.eq(account_id))
			.select((accounts::all_columns))
			.first::<Account>(conn)
			.map_err(Into::into)
	}
	
	pub fn transact(&self, k: BankTransactionType, account_id: &uuid::Uuid, value: &BigDecimal) ->
	Result<Account> {
		let conn = &self.db.get()?;
		let neg_value = value.neg();
		let v = match k {
			BankTransactionType::Deposit => value,
			BankTransactionType::Withdraw => &neg_value,
		};
		
		diesel::update(accounts::table)
			.filter(accounts::id.eq(account_id))
			.set(accounts::amount.eq(accounts::amount + v))
			.get_result(conn)
			.map_err(Into::into)
	}
}

