from database import get_db_connection, close_connection

def create_table():
    connection = get_db_connection()
    if connection:
        try:
            cursor = connection.cursor()
            
            # Create a sample table
            create_table_query = '''
                CREATE TABLE IF NOT EXISTS users (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(100) NOT NULL,
                    email VARCHAR(100) UNIQUE NOT NULL
                )
            '''
            cursor.execute(create_table_query)
            
            # Commit the changes
            connection.commit()
            print("Table created successfully")
            
        except Exception as e:
            print(f"Error: {e}")
        finally:
            cursor.close()
            close_connection(connection)

def insert_user(name, email):
    connection = get_db_connection()
    if connection:
        try:
            cursor = connection.cursor()
            
            # Insert a new user
            insert_query = '''
                INSERT INTO users (name, email)
                VALUES (%s, %s)
                RETURNING id
            '''
            cursor.execute(insert_query, (name, email))
            user_id = cursor.fetchone()[0]
            
            # Commit the changes
            connection.commit()
            print(f"User inserted successfully with ID: {user_id}")
            
        except Exception as e:
            print(f"Error: {e}")
        finally:
            cursor.close()
            close_connection(connection)

def get_all_users():
    connection = get_db_connection()
    if connection:
        try:
            cursor = connection.cursor()
            
            # Select all users
            select_query = "SELECT * FROM users"
            cursor.execute(select_query)
            users = cursor.fetchall()
            
            for user in users:
                print(f"ID: {user[0]}, Name: {user[1]}, Email: {user[2]}")
            
        except Exception as e:
            print(f"Error: {e}")
        finally:
            cursor.close()
            close_connection(connection)

if __name__ == "__main__":
    # Create the table
    create_table()
    
    # Insert some sample users
    insert_user("John Doe", "john@example.com")
    insert_user("Jane Smith", "jane@example.com")
    
    # Retrieve and display all users
    get_all_users()
