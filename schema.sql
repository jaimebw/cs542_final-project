-- This file outlines the base structure of the test sqlite database. The test database will be recreated when this
-- file is changed.

CREATE TABLE Site_users
(
    sid           BINARY(16),
    email         VARCHAR(100),
    password_hash BINARY(32),
    UNIQUE(email),--add UNIQUE(email)
    PRIMARY KEY (sid)
);

CREATE TABLE Department
(
    DepID BINARY(16),
    name  VARCHAR(255) UNIQUE,
    PRIMARY KEY (DepID)
);

CREATE TABLE Manufacturer
(
    ManuID BINARY(16),
    name   VARCHAR(255) UNIQUE,
    PRIMARY KEY (ManuID)
);

CREATE TABLE Sold_Product_Manufactured
(
    PID    BINARY(16),
    URL    VARCHAR(1024),
    name   VARCHAR(1024),
    DepID  BINARY(16) NOT NULL,
    ManuID BINARY(16) NOT NULL,
    Primary Key (PID),
    UNIQUE(URL),--add UNIQUE(URL)
    Foreign Key (DepID) REFERENCES Department (DepID),
    Foreign Key (ManuID) REFERENCES Manufacturer (ManuID)
);

CREATE TABLE Tracks
(
    sid BINARY(16),
    PID BINARY(16),
    PRIMARY KEY (sid, PID),
    FOREIGN KEY (sid) REFERENCES Site_users (sid),
    FOREIGN KEY (PID) REFERENCES Sold_Product_Manufactured (PID)
);

CREATE TABLE Product_variant_Sold
(
    ASIN      CHAR(10),
    variation VARCHAR(255),
    type      VARCHAR(255),
    PID       BINARY(16) NOT NULL,
    PRIMARY KEY (ASIN),
    FOREIGN KEY (PID) REFERENCES Sold_Product_Manufactured (PID)ON DELETE CASCADE--add on delete cascade
);



CREATE TABLE Deal_Alert_on
(
    conditions        CHAR(20),
    ASIN              CHAR(10),
    last_notification CHAR(20),
    Primary Key (conditions, ASIN),
    FOREIGN Key (ASIN) REFERENCES Product_variant_Sold (ASIN) ON DELETE CASCADE
);


CREATE TABLE Subscribes_To
(
    conditions CHAR(20),
    ASIN       CHAR(10),
    sid        BINARY(16),
    Primary Key (conditions, ASIN, sid),
    Foreign Key (ASIN) REFERENCES Product_variant_Sold (ASIN) ON DELETE CASCADE,
    FOREIGN Key (conditions, ASIN) REFERENCES Deal_Alert_on (conditions, ASIN) ON DELETE CASCADE,
    FOREIGN Key (sid) REFERENCES Site_users (sid)
);

CREATE TABLE Area_within
(
    sub_DepID      BINARY(16),
    Category_DepID BINARY(16),
    Primary Key (sub_DepID, Category_DepID),
    FOREIGN Key (sub_DepID) REFERENCES Department (DepID),
    FOREIGN Key (Category_DepID) REFERENCES Department (DepID)
);

CREATE TABLE Contains_Reviews
(
    ASIN   CHAR(10),--change amazonID to ASIN
    PID        BINARY(16),
    rating     REAL,
    reviewdate DATE,
    Primary Key (ASIN, PID),
    FOREIGN Key (ASIN) REFERENCES Product_variant_Sold (ASIN) ON DELETE CASCADE,
    FOREIGN KEY (PID) REFERENCES Sold_Product_Manufactured (PID) ON DELETE CASCADE
);

CREATE TABLE Ranked_Best_Seller_Rank
(
    rank     CHAR(20),
    ASIN     CHAR(10),
    category CHAR(20),
    Primary Key (rank, ASIN),
    FOREIGN KEY (ASIN) REFERENCES Product_variant_Sold (ASIN) ON DELETE CASCADE
);

CREATE TABLE For_Product_Data_Refresh
(
    datetime date,
    ASIN     CHAR(10),
    Primary Key (datetime,ASIN),
    Foreign Key (ASIN) REFERENCES Product_variant_Sold (ASIN) ON DELETE CASCADE
);

CREATE TABLE Company
(
    ComID BINARY(16),
    name  CHAR(20),
    Primary Key (ComID)
);

CREATE TABLE Has_Listing_collected
(
    ListingID     BINARY(16),
    ASIN          CHAR(10),
    condition     CHAR(20),
    Price         real,
    datetime      date     NOT NULL,
    shipped_comID BINARY(16) NOT NULL,
    sold_ComID    BINARY(16) NOT NULL,
    Primary Key (ListingID, ASIN),
    Foreign Key (ASIN) REFERENCES Product_variant_Sold (ASIN) ON DELETE CASCADE,
    Foreign Key (ASIN,datetime) REFERENCES For_Product_Data_Refresh (ASIN,datetime) ON DELETE CASCADE,
    Foreign Key (shipped_comID) REFERENCES Company (ComID),
    Foreign Key (sold_ComID) REFERENCES Company (ComID)
);
