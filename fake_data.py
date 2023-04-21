import sqlite3
from datetime import timedelta
from faker import Faker
import secrets
import hashlib
import random
import string
from pathlib import Path
import os


Faker.seed(42)
NUMBER_OF_USERS = 5
N_PRODUCTS = 5
N_DAYS = 5


def create_db():
    if Path("local.sqlite").exists():
        os.remove("local.sqlite")
    conn = sqlite3.connect("local.sqlite")
    with open("schema.sql") as f:
        schema = f.read()
        conn.executescript(schema)
    return conn 

def generate_asin():
    # Amazon ASINs are 10 characters long
    length = 10
    
    # Amazon ASINs can contain uppercase letters, numbers, and hyphens
    characters = string.ascii_uppercase + string.digits + '-'
    
    # Generate a random string of characters
    asin = ''.join(random.choice(characters) for i in range(length))
    
    # Return the ASIN
    return asin


def random_id():
    # Generate a random 16-byte (128-bit) binary ID
    binary_id = secrets.token_bytes(16)
    return binary_id


def hash_password(password):
    SALT = bytearray([242, 94, 145, 122, 201, 1, 131, 203])
    hasher = hashlib.sha256()
    hasher.update(SALT)
    hasher.update(password.encode())

    return hasher.digest()


# Example usage

fake = Faker()

conn = create_db()
cur = conn.cursor()

for i in range(NUMBER_OF_USERS):
    # Site_users
    sid = random_id()
    email = fake.email()
    password = fake.password()
    password_hash= hash_password(password)
    print(email,password)
    site_users = [sid, email, password_hash]
    cur.execute("INSERT INTO Site_users (sid, email, password_hash) VALUES (?,?,?) ",
            site_users)
    # Deparment
    depid = random_id()
    name = fake.name()
    departments = [depid,name]
    cur.execute("INSERT INTO Department(DepID, name) VALUES (?,?) ",
            departments)
    # Manufacturer
    manuid = random_id()
    man_name = fake.company()
    manufacturers = [manuid,man_name]
    cur.execute("INSERT INTO Manufacturer(ManuID, name) VALUES (?,?) ",
            manufacturers)
    # Sold_Product_Manufactured
    pid = random_id()
    url = fake.url()
    product_name = " ".join(fake.words(nb=2, unique=True))
    sold_product_manufactured = [
        pid,
        url,
        product_name,
        depid,
        manuid
            ]
    cur.execute("INSERT INTO Sold_Product_Manufactured(PID,URL,name,DepID,ManuID) \
            VALUES (?,?,?,?,?)",sold_product_manufactured)

    # Tracks
    cur.execute("INSERT INTO Tracks(sid,PID) \
            VALUES (?,?)",[sid,pid])

    #Productt variants sold_product_manufactured
    asin = generate_asin()
    print(asin)
    variation = random.choice(["first_var","second_var","none"])
    ttype = "none"
    product_variant_sold = [
            asin,
            variation,
            ttype,
            pid
            ]
    cur.execute("INSERT INTO product_variant_sold(ASIN,variation,type,PID) \
            VALUES (?,?,?,?)",product_variant_sold)
    # Deal Alert on
    # Change conditions to be random
    conditions = random.choice(["Good","Bad","Excellent","Fair"])
    last_notification = fake.date()
    deal_alerts_on = [conditions,asin,last_notification]
    cur.execute("INSERT INTO Deal_Alert_on(conditions,ASIN,last_notification)\
            VALUES (?,?,?)",deal_alerts_on)
    # Subscribes_To
    subscribes_to = [
            conditions,
            asin,
            sid
            ]
    cur.execute("INSERT INTO Subscribes_To(conditions,ASIN,sid) \
            VALUES (?,?,?)", subscribes_to)
    # Area withing
    cur.execute("INSERT INTO Area_within(sub_DepID,Category_DepID) \
            VALUES (?,?)", [depid,depid])
    # Contains reviews
    rating = random.randint(1, 5) 
    review_date  = fake.date_between(start_date='-3m', end_date='today')
    contains_reviews = [asin,pid,rating,review_date]
    cur.execute("INSERT INTO Contains_Reviews(ASIN,PID,rating,reviewdate)\
            VALUES (?,?,?,?)",contains_reviews)
    
    # Ranked best seller rank
    rank = str(random.randint(1,5)) # This might be wrong in the sql schema
    category = random.choice(["House","Videogames","Kitchen"])
    ranked_best = [rank,asin,category]
    cur.execute("INSERT INTO Ranked_Best_Seller_Rank(rank,ASIN,category) \
            VALUES (?,?,?)", ranked_best)
    # Company
    comid = random_id()
    comp_name = fake.company()
    company = [comid,comp_name]
    cur.execute("INSERT INTO Company(ComID,name) \
                VALUES (?,?)", company)

    conn.commit()
    # For_Product_Data_Refres
    datetime0 =  fake.date_between(start_date='-1m', end_date='today')
    for day in range(N_DAYS):
        if day == 0:
           datetime = datetime0 
        else:
            datetime = datetime0 + timedelta(days = day) 
        print(f"Date:{datetime}, asin:{asin}")

        cur.execute("INSERT INTO For_Product_Data_Refresh(datetime,ASIN) \
                VALUES (?,?)", [datetime,asin])
        # Has_Listing_collected
        listing_id = random_id()
        price = round(random.uniform(2,50))
        has_listing_collected = [
                listing_id,
                asin,
                conditions,
                price,
                datetime,
                comid,
                comid
                ]

        cur.execute("INSERT INTO Has_Listing_collected(ListingID,ASIN,condition,\
                Price,datetime,shipped_comID,sold_ComID) VALUES (?,?,?,?,?,?,?)",has_listing_collected)
        conn.commit()





