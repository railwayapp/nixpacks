import requests

def fetch_data():
    response = requests.get("https://jsonplaceholder.typicode.com/todos/1")
    if response.status_code == 200:
        print("Data fetched successfully!")
        print(response.json())
    else:
        print("Failed to fetch data.")

if __name__ == "__main__":
    fetch_data()
