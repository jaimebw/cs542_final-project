import sqlite3
from faker import Faker
import secrets
import binascii
import hashlib
import random
import string

Faker.seed(42)

## WIP

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
    # Convert the binary ID to a hexadecimal string
    hex_id = binascii.hexlify(binary_id).decode('utf-8')
    return hex_id


def hash_password(password):
    SALT = bytearray([242, 94, 145, 122, 201, 1, 131, 203])

    hasher = hashlib.sha256()
    hasher.update(SALT)
    hasher.update(password.encode())

    return hasher.digest()

# Example usage

fake = Faker()

conn = sqlite3.connect('local.sqlite')
cur = conn.cursor()

for i in range(10):
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
    man_name = fake.name()
    manufacturers = [manuid,man_name]
    cur.execute("INSERT INTO Manufacturer(ManuID, name) VALUES (?,?) ",
            manufacturers)
    # Sold_Product_Manufactured
    pid = random_id()
    url = fake.url()
    product_name = fake.name()
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
    conditions = "none" # idk what to put here
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
    cur.execute("INSERT INTO Are_within(sub_DepID,Category_DepID) \
            VALUES (?,?)", [depid,depid])
    # Contains reviews



    conn.commit()





