use std::sync::Mutex;

use crate::models::{CountGroupByStatus, TrainJobData, TrainJobDetails, TrainJobResponseData};
use crate::{database, runtime_err::RunTimeError};
use rusqlite::{Connection, Result};

pub fn save(conn: &Mutex<Connection>, job_data: TrainJobData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "INSERT INTO train_job (group_id, account_id, like_probable, floow_probable, collect_probable, status,start_time) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            job_data.group_id,
            job_data.account_id,
            job_data.like_probable,
            job_data.floow_probable,
            job_data.collect_probable,
            job_data.status,
            job_data.start_time,
        ],
    )?;
    Ok(())
}
pub fn update(conn: &Mutex<Connection>, job_data: TrainJobData) -> Result<(), RunTimeError> {
    let _lock = conn.lock();
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE train_job SET status = ?1 WHERE id = ?2",
        rusqlite::params![job_data.status, job_data.id,],
    )?;
    if job_data.status.unwrap() == 2 {
        //update end_time
        let end_time = chrono::Local::now().naive_local();
        //convert to string
        let end_time = end_time.format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE train_job SET end_time = ?1 WHERE id = ?2",
            rusqlite::params![end_time, job_data.id,],
        )?;
    }
    Ok(())
}
pub fn list_all() -> Result<TrainJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT train_job.id,train_job.group_id,train_job.account_id,
    train_job.like_probable,train_job.floow_probable,train_job.collect_probable,train_job.status,
    train_job.start_time,train_job.end_time,account.device,account.username FROM train_job
    left join account on train_job.account_id = account.id
    ORDER BY train_job.id DESC LIMIT 200
    ",
    )?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map((), |row| {
        Ok(TrainJobDetails {
            id: row.get(0)?,
            group_id: row.get(1)?,
            account_id: row.get(2)?,
            like_probable: row.get(3)?,
            floow_probable: row.get(4)?,
            collect_probable: row.get(5)?,
            status: row.get(6)?,
            start_time: row.get(7)?,
            end_time: row.get(8)?,
            device: row.get(9)?,
            username: row.get(10)?,
        })
    })?;
    for job in job_iter {
        data.push(job?);
    }
    Ok(TrainJobResponseData { data })
}
pub fn del(id: i32) -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute("DELETE FROM train_job WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
pub fn list_runable(agent_ip: String) -> Result<TrainJobResponseData, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT train_job.id,train_job.group_id,train_job.account_id,
    train_job.like_probable,train_job.floow_probable,train_job.collect_probable,train_job.status,
    train_job.start_time,train_job.end_time,account.device,account.username FROM train_job
    left join account on train_job.account_id = account.id
    left join device on account.device = device.serial
    WHERE train_job.status = 0 AND device.agent_ip = ?1 
    AND train_job.start_time < datetime('now', 'localtime') 
    AND device.online = 1
    ORDER BY train_job.id ASC
    ",
    )?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map(rusqlite::params![agent_ip], |row| {
        Ok(TrainJobDetails {
            id: row.get(0)?,
            group_id: row.get(1)?,
            account_id: row.get(2)?,
            like_probable: row.get(3)?,
            floow_probable: row.get(4)?,
            collect_probable: row.get(5)?,
            status: row.get(6)?,
            start_time: row.get(7)?,
            end_time: row.get(8)?,
            device: row.get(9)?,
            username: row.get(10)?,
        })
    })?;
    for publish_job in job_iter {
        data.push(publish_job?);
    }
    Ok(TrainJobResponseData { data })
}

pub fn count_job_by_account_today(
    account_id: i32,
    start_time: String,
) -> Result<i32, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT count(*) FROM train_job
    WHERE account_id = ?1 AND start_time = ?2 AND DATE(create_time) = DATE('now')
    ",
    )?;
    let mut count = 0;
    let job_iter = stmt.query_map(rusqlite::params![account_id, start_time], |row| {
        Ok(row.get(0)?)
    })?;
    for job in job_iter {
        count = job?;
    }
    Ok(count)
}

pub fn count_by_status() -> Result<Vec<CountGroupByStatus>, RunTimeError> {
    let conn = database::get_conn()?;
    let mut stmt = conn.prepare(
        "
    SELECT status,count(*) FROM train_job
    GROUP BY status
    ",
    )?;
    let mut data = Vec::new();
    let job_iter = stmt.query_map((), |row| {
        Ok(CountGroupByStatus {
            status: row.get(0)?,
            count: row.get(1)?,
        })
    })?;
    for job in job_iter {
        data.push(job?);
    }
    Ok(data)
}
pub fn retry_all_failed() -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    conn.execute(
        "UPDATE train_job SET status = 0 WHERE status = 3",
        rusqlite::params![],
    )?;
    Ok(())
}
pub fn delete_all() -> Result<(), RunTimeError> {
    let conn = database::get_conn()?;
    //truncate table
    conn.execute("DELETE FROM train_job", rusqlite::params![])?;
    //reset autoincrement
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name='train_job'",
        rusqlite::params![],
    )?;
    Ok(())
}
