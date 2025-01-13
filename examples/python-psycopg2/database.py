import os
import psycopg2
from psycopg2 import Error
import time

def get_db_connection():
    max_retries = 3
    retry_delay = 2  # seconds
    
    for attempt in range(max_retries):
        try:
            connection = psycopg2.connect(
                host=os.getenv('PGHOST'),
                database=os.getenv('PGDATABASE'),
                user=os.getenv('PGUSER'),
                password=os.getenv('PGPASSWORD'),
                port=os.getenv('PGPORT')
            )
            return connection
        except Error as e:
            if "starting up" in str(e).lower():
                if attempt < max_retries - 1:  # Don't sleep on last attempt
                    time.sleep(retry_delay)
                    continue
            print(f"Error connecting to PostgreSQL: {e}")
            return None

def close_connection(connection):
    if connection:
        connection.close()
