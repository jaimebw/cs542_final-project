-- This file outlines the base structure of the test sqlite database. The test database will be recreated when this
-- file is changed.

CREATE TABLE Site_users(
sid CHAR(20),
email CHAR(20),
password_hash CHAR(20),
PRIMARY KEY(sid));

CREATE TABLE Department(
DepID CHAR(20),
name CHAR(20),
PRIMARY KEY(DepID));

CREATE TABLE Manufacturer(
ManuID CHAR(20),
name CHAR(20),
PRIMARY KEY(ManuID));

CREATE TABLE Sold_Product_Manufactured(
PID CHAR(20),
URL CHAR(20),
name CHAR(20), 
DepID CHAR(20)NOT NULL, 
ManuID CHAR(20)NOT NULL,
Primary Key(PID),
Foreign Key (DepID) REFERENCES Department(DepID),
Foreign Key (ManuID) REFERENCES Manufacturer(ManuID));

CREATE TABLE Tracks(
sid CHAR(20),
PID CHAR(20),
PRIMARY KEY(sid,PID),
FOREIGN KEY(sid) REFERENCES Site_users (sid),
FOREIGN KEY(PID) REFERENCES Sold_Product_Manufactured(PID));

CREATE TABLE Product_variant_Sold(
ASIN CHAR(20),
variation CHAR(20),
type CHAR(20),
PID CHAR(20)NOT NULL,
PRIMARY KEY(ASIN),
FOREIGN KEY(PID) REFERENCES Sold_Product_Manufactured(PID));



CREATE TABLE Deal_Alert_on(
conditions CHAR(20),
ASIN CHAR(20),
last_notification CHAR(20),
Primary Key(conditions, ASIN),
FOREIGN Key(ASIN) REFERENCES Product_variant_Sold(ASIN)ON DELETE CASCADE);


CREATE TABLE Subscribes_To(
conditions CHAR(20),
ASIN CHAR(20),
sid CHAR(20),
Primary Key(conditions,ASIN,sid),
FOREIGN Key(conditions,ASIN)REFERENCES Deal_Alert_on(conditions,ASIN),
FOREIGN Key(sid)REFERENCES Site_users(sid));

CREATE TABLE Area_within(
sub_DepID CHAR(20),
Category_DepID CHAR(20),
Primary Key(sub_DepID, Category_DepID),
FOREIGN Key(sub_DepID)REFERENCES Department(DepID),
FOREIGN Key(Category_DepID) REFERENCES Department(DepID));

CREATE TABLE Contains_Reviews(
amazonID CHAR(20),
PID CHAR(20),
rating REAL, 
reviewdate DATE,
Primary Key( amazonID, PID),
FOREIGN KEY(PID) REFERENCES Sold_Product_Manufactured (PID)ON DELETE CASCADE);

CREATE TABLE Ranked_Best_Seller_Rank(
rank CHAR(20),
ASIN CHAR(20),
category CHAR(20),
Primary Key( rank, ASIN),
FOREIGN KEY (ASIN)REFERENCES Product_variant_Sold(ASIN)ON DELETE CASCADE);

CREATE TABLE For_Product_Data_Refresh(
datetime date, 
ASIN CHAR(20),
Primary Key(datetime),
Foreign Key(ASIN)REFERENCES Product_variant_Sold(ASIN));

CREATE TABLE Company(
ComID CHAR(20),
name CHAR(20),
Primary Key(ComID));

CREATE TABLE Has_Listing_collected(
ListingID CHAR(20),
ASIN CHAR(20),
condition CHAR(20),
Price real, 
datetime date NOT NULL,
shipped_comID CHAR(20) NOT NULL, 
sold_ComID CHAR(20)NOT NULL,
Primary Key(ListingID, ASIN),
Foreign Key(ASIN)REFERENCES Product_variant_Sold(ASIN)ON DELETE CASCADE,
Foreign Key(datetime)REFERENCES For_Product_Data_Refresh(datetime),
Foreign Key (shipped_comID)REFERENCES Company (ComID),
Foreign Key (sold_ComID)REFERENCES Company (ComID));
