use mysql;
use mysql::prelude::Queryable;
use mysql::{params, Opts, Pool};

type MySqlResult<T> = std::result::Result<T, mysql::Error>;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Row {
    pub id: i32,
    pub number: i32,
    pub text: Option<String>,
}

impl Into<Vec<String>> for &Row {
    fn into(self) -> Vec<String> {
        vec![
            self.number.to_string(),
            self.text.to_owned().unwrap_or_default(),
        ]
    }
}

pub struct Database {
    url: String,
    pool: Pool,
}

impl Database {
    pub fn new(url: String) -> MySqlResult<Self> {
        let pool = Pool::new(url.clone())?;
        pool.get_conn()?;
        Ok(Database {
            url: url,
            pool: pool,
        })
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn set_url(&mut self, url: String) -> MySqlResult<()> {
        let pool = Pool::new(Opts::from_url(&url)?)?;
        pool.get_conn()?;
        self.url = url;
        Ok(())
    }

    pub fn create_table(&self) -> MySqlResult<()> {
        let mut connection = self.pool.get_conn()?;

        connection.query_drop(r"DROP TABLE IF EXISTS simple_table")?;
        connection.query_drop(r"FLUSH TABLES")?;

        connection.query_drop(
            r"CREATE TABLE simple_table (
                id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                number INTEGER,
                text TEXT
            )",
        )?;

        connection.query_drop(r"FLUSH TABLES")?;

        Ok(())
    }

    pub fn _drop_table(&self) -> MySqlResult<()> {
        let mut connection = self.pool.get_conn()?;
        connection.query_drop(r"DROP TABLE IF EXISTS simple_table")?;
        Ok(())
    }

    pub fn get_rows(&self) -> MySqlResult<Vec<Row>> {
        let mut connection = self.pool.get_conn().unwrap();

        Ok(connection
            .query_map(
                r"SELECT id, number, text FROM simple_table",
                |(id, number, text)| Row { id, number, text },
            )
            .unwrap())
    }

    pub fn insert_rows(&self, rows: Vec<Row>) -> MySqlResult<()> {
        let mut connection = self.pool.get_conn()?;

        connection.exec_batch(
            r"INSERT INTO simple_table (number, text)
            VALUES (:number, :text)",
            rows.iter().map(|row| {
                params! {
                    "number" => row.number,
                    "text" => &row.text,
                }
            }),
        )
    }

    pub fn insert_row(&self, row: Row) -> MySqlResult<()> {
        let mut connection = self.pool.get_conn()?;

        connection.exec_drop(
            r"INSERT INTO simple_table (number, text)
            VALUES (:number, :text)",
            params! {
                "number" => row.number,
                "text" => &row.text,
            },
        )
    }

    pub fn update_number(&self, row: usize, new_value: i32) -> MySqlResult<()> {
        let mut connection = self.pool.get_conn()?;

        connection.exec_drop(
            r"UPDATE simple_table SET number = :number WHERE id = :row;",
            params! {
                "number" => new_value,
                "row" => row,
            },
        )
    }

    pub fn update_text(&self, row: usize, new_value: Option<String>) -> MySqlResult<()> {
        let mut connection = self.pool.get_conn()?;

        connection.exec_drop(
            r"UPDATE simple_table SET text = :text WHERE id = :row;",
            params! {
                "text" => new_value,
                "row" => row,
            },
        )
    }
}

pub fn populate_table(db: &Database) {
    db.create_table().unwrap();

    db.insert_rows(vec![
        Row {
            id: 0,
            number: 1,
            text: Some("test".to_owned()),
        },
        Row {
            id: 0,
            number: 100,
            text: Some("another text".to_owned()),
        },
        Row {
            id: 0,
            number: -3234,
            text: None,
        },
    ])
    .unwrap();
}

#[cfg(test)]
mod tests {
    use crate::database::{Database, Row};
    use std::sync::Once;

    static INIT: Once = Once::new();
    static mut DATABASE: Option<Database> = None;

    fn get_cached_database() -> &'static Database {
        unsafe {
            INIT.call_once(|| {
                DATABASE = Some(
                    Database::new("mysql://zotho:zotho@localhost:3306/rust".to_owned()).unwrap(),
                );
            });
            DATABASE.as_ref().unwrap()
        }
    }

    #[test]
    fn test_database_insert_get() {
        let db: &Database = get_cached_database();
        db.create_table().unwrap();

        let test_rows = vec![
            Row {
                id: 1,
                number: 1,
                text: Some("test".to_owned()),
            },
            Row {
                id: 2,
                number: 100,
                text: Some("another text".to_owned()),
            },
            Row {
                id: 3,
                number: -3234,
                text: None,
            },
        ];
        db.insert_rows(test_rows.clone()).unwrap();

        let actual_rows = db.get_rows().unwrap();
        assert_eq!(actual_rows, test_rows);

        db._drop_table().unwrap();
    }

    #[test]
    fn test_database_update() {
        let db: &Database = get_cached_database();
        db.create_table().unwrap();

        let mut test_rows = vec![
            Row {
                id: 1,
                number: 1,
                text: Some("test".to_owned()),
            },
            Row {
                id: 2,
                number: 100,
                text: Some("another text".to_owned()),
            },
            Row {
                id: 3,
                number: -3234,
                text: None,
            },
        ];
        db.insert_rows(test_rows.clone()).unwrap();

        db.update_number(1, 2).unwrap();
        db.update_text(3, Some("TEST".to_owned())).unwrap();

        let actual_rows = db.get_rows().unwrap();

        test_rows[0].number = 2;
        test_rows[2].text = Some("TEST".to_owned());
        assert_eq!(actual_rows, test_rows);

        db._drop_table().unwrap();
    }
}
