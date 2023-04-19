use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::{Request, State};
use sqlx::pool::PoolConnection;
use sqlx::{Executor, Pool, Sqlite};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;
use crate::scraper::product::{DepartmentHierarchy, Product};
use crate::session::UserId;

/// A database connection that can be used in routes to acquire a database handle
#[repr(transparent)]
pub struct Connection<D: sqlx::Database> {
    connection: PoolConnection<D>,
}

#[rocket::async_trait]
impl<'r, D: sqlx::Database> FromRequest<'r> for Connection<D> {
    type Error = Option<sqlx::Error>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.guard::<&State<Pool<D>>>().await {
            Success(x) => match x.acquire().await {
                Ok(connection) => Success(Connection { connection }),
                Err(err) => Failure((Status::ServiceUnavailable, Some(err))),
            },
            Failure((status, ())) => Failure((status, None)),
            Forward(()) => Forward(()),
        }
    }
}

impl<D: sqlx::Database> Deref for Connection<D> {
    type Target = PoolConnection<D>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<D: sqlx::Database> DerefMut for Connection<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

impl Connection<Sqlite> {

    pub async fn department_by_name(&mut self, department: &str) -> sqlx::Result<Option<Uuid>> {
        Ok(sqlx::query_as("SELECT DepID FROM Department WHERE name = ?")
            .bind(department)
            .fetch_optional(&mut self.connection)
            .await?
            .map(|(uuid, )| uuid))
    }

    pub async fn get_or_add_department(&mut self, department_hierarchy: &DepartmentHierarchy) -> sqlx::Result<Uuid> {
        let last_department = match department_hierarchy.last() {
            Some(last) => last,
            None => unreachable!("Department hierarchy must contain at least one item")
        };

        // Check if the department is already present before going through the process to add it
        if let Some(id) = self.department_by_name(&last_department.name).await? {
            return Ok(id)
        }

        // Add departments until we
        let mut parent = None;
        let mut maybe_initialized = true;

        for department in department_hierarchy.iter() {
            if maybe_initialized {
                if let Some(id) = self.department_by_name(&last_department.name).await? {
                    parent = Some(id);
                    continue
                } else {
                    maybe_initialized = false;
                }
            }

            let new_department = Uuid::new_v4();
            sqlx::query("INSERT INTO Department (DepID, name) VALUES (?, ?)")
                .bind(new_department)
                .bind(&department.name)
                .execute(&mut self.connection)
                .await?;

            if let Some(parent_department) = parent {
                sqlx::query("INSERT INTO Area_within (sub_DepID, Category_DepID) VALUES (?, ?)")
                    .bind(new_department)
                    .bind(parent_department)
                    .execute(&mut self.connection)
                    .await?;
            }

            parent = Some(new_department);
        }

        Ok(parent.unwrap())
    }

    pub async fn product_exists(&mut self, asin: &str) -> sqlx::Result<Option<Uuid>> {
        Ok(sqlx::query_as("SELECT PID FROM Product_variant_Sold WHERE ASIN = ?")
            .bind(asin)
            .fetch_optional(&mut self.connection)
            .await?
            .map(|(id,)| id))
    }

    pub async fn get_or_add_manufacturer(&mut self, name: &str) -> sqlx::Result<Uuid> {
        let current_id = sqlx::query_as("SELECT ManuID FROM Manufacturer WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut self.connection)
            .await?;

        if let Some((id,)) = current_id {
            return Ok(id)
        }

        let new_id = Uuid::new_v4();
        sqlx::query("INSERT INTO Manufacturer (ManuID, name) VALUES (?, ?)")
            .bind(new_id)
            .bind(name)
            .execute(&mut self.connection)
            .await?;

        Ok(new_id)
    }

    pub async fn add_product(&mut self, product: &Product) -> sqlx::Result<Uuid> {
        let manufacturer_id = self.get_or_add_manufacturer(&product.manufacturer).await?;
        let department_id = self.get_or_add_department(&product.department).await?;

        let url  = format!("https://amazon.com/dp/{}/", &product.asin);

        let new_id = Uuid::new_v4();
        sqlx::query("INSERT INTO Sold_Product_Manufactured (PID, URL, name, DeptID, ManuID) VALUES (?, ?, ?, ?, ?)")
            .bind(new_id)
            .bind(url)
            .bind(&product.name)
            .bind(department_id)
            .bind(manufacturer_id)
            .execute(&mut self.connection)
            .await?;

        sqlx::query("INSERT INTO Product_variant_Sold (ASIN, variation, type, PID) VALUES (?, ?, ?, ?)")
            .bind(&product.asin)
            .bind("default")
            .bind("")
            .bind(new_id)
            .execute(&mut self.connection)
            .await?;

        Ok(new_id)
    }

    pub async fn track_product(&mut self, user: UserId, product: Uuid) -> sqlx::Result<()> {
        let (exits,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM Tracks WHERE sid = ? AND PID = ?)")
            .bind(&user)
            .bind(product)
            .fetch_one(&mut self.connection)
            .await?;

        if !exits {
            sqlx::query("INSERT INTO Tracks (sid, PID) VALUES (?, ?)")
                .bind(user)
                .bind(product)
                .execute(&mut self.connection)
                .await?;
        }

        Ok(())
    }
}
