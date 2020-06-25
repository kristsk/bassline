pub struct Bassline<'conn> {
    get_random_write_statement: rusqlite::Statement<'conn>,
    add_write_statement: rusqlite::Statement<'conn>,
    add_read_log_statement: rusqlite::Statement<'conn>,
    db_connection: &'conn rusqlite::Connection,
}

pub struct Write {
    pub id: i64,
    pub created_by: String,
    pub content: String,
}

impl<'conn> Bassline<'conn> {
    pub fn new(db_connection: &'conn rusqlite::Connection) -> Result<Bassline, String> {
        Bassline::check_database(&db_connection)?;
        Ok(Bassline {
            get_random_write_statement: db_connection.prepare(include_str!("../database/get_random_write.sql")).unwrap(),
            add_write_statement: db_connection.prepare(include_str!("../database/add_write.sql")).unwrap(),
            add_read_log_statement: db_connection.prepare(include_str!("../database/add_read_log.sql")).unwrap(),
            db_connection,
        })
    }

    fn check_database(db_connection: &rusqlite::Connection) -> Result<(), String> {
        if !Bassline::table_exists(db_connection, "writes")? {
            Bassline::create_database(db_connection).unwrap();
        }
        Ok(())
    }

    fn table_exists(db_connection: &rusqlite::Connection, name: &str) -> Result<bool, String> {
        match db_connection.query_row(include_str!("../database/table_exists.sql"), &[&name], |row| row.get(0)) {
            Ok(0) => Ok(false),
            Ok(_) => Ok(true),
            Err(error) => Err(error.to_string())
        }
    }

    fn create_database(db_connection: &rusqlite::Connection) -> Result<(), String> {
        match db_connection.execute_batch(include_str!("../database/create_database.sql")) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn get_random_write(&mut self, reader_nickname: &str) -> Result<Write, String> {
        let result = self.get_random_write_statement.query_row(
            rusqlite::NO_PARAMS,
            |row| Ok(Write {
                id: row.get(0).unwrap(),
                created_by: row.get(1).unwrap(),
                content: row.get(2).unwrap(),
            }),
        );
        match result {
            Ok(write) => {
                let write_id_string = write.id.to_string();
                let log_params = [Some(write_id_string.as_str()), Some(reader_nickname)];
                match self.add_read_log_statement.execute(&log_params) {
                    Ok(_) => Ok(write),
                    Err(error) => Err(error.to_string())
                }
            }
            Err(error) => Err(error.to_string())
        }
    }

    pub fn add_write(&mut self, nickname: &str, content: &str, source: &str) -> Result<i64, String> {
        let params = [Some(nickname), Some(content), Some(source)];
        match self.add_write_statement.execute(&params) {
            Ok(_) => {
                Ok(self.db_connection.last_insert_rowid())
            }
            Err(error) => Err(error.to_string())
        }
    }

    pub fn respond_to_read(&mut self, reader_nickname: &str) -> Result<String, String> {
        let write = self.get_random_write(reader_nickname)?;
        Ok(format!("{}: {}", write.created_by, write.content))
    }

    pub fn respond_to_write(&mut self, nickname: &str, content: &str, source: &str) -> Result<String, String> {
        let write_id = self.add_write(nickname, content, source)?;
        Ok(format!("Pierakstiiju, do veel! (#{})", write_id))
    }
}
