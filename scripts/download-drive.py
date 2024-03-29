import os.path
import io


from pathlib import Path



from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
from googleapiclient.http import MediaIoBaseDownload

if (not os.path.exists(f"{str(Path.home())}/Images")):
    os.mkdir(f"{str(Path.home())}/Images")
if (not os.path.exists(f"{str(Path.home())}/Images/steg")):
    os.mkdir(f"{str(Path.home())}/Images/steg")
if (not os.path.exists(f"{str(Path.home())}/Images/steg/base_img")):
    os.mkdir(f"{str(Path.home())}/Images/steg/base_img")

# If modifying these scopes, delete the file token.json.
SCOPES = ["https://www.googleapis.com/auth/drive"]


def main():
  """Shows basic usage of the Drive v3 API.
  Prints the names and ids of the first 10 files the user has access to.
  """
  creds = None
  # The file token.json stores the user's access and refresh tokens, and is
  # created automatically when the authorization flow completes for the first
  # time.
  if os.path.exists("token.json"):
    creds = Credentials.from_authorized_user_file("token.json", SCOPES)
  # If there are no (valid) credentials available, let the user log in.
  if not creds or not creds.valid:
    if creds and creds.expired and creds.refresh_token:
      creds.refresh(Request())
    else:
      flow = InstalledAppFlow.from_client_secrets_file(
          "credentials.json", SCOPES
      )
      creds = flow.run_local_server(port=0)
    # Save the credentials for the next run
    with open("token.json", "w") as token:
      token.write(creds.to_json())

  try:
    service = build("drive", "v3", credentials=creds)

    # Call the Drive v3 API
    results = (
        service.files()
        .list(q="mimeType != 'application/vnd.google-apps.folder' and '1DLMMbazkkRxCF7s8EZ3scStwY6jKxjKM' in parents", pageSize=100, fields="nextPageToken, files(id, name)")
        .execute()
    )
    items = results.get("files", [])
    if not items:
      print("No files found.")
      return
    print("Files:")
    for item in items:
        print(f"{item['name']} ({item['id']})")
        file_name=f"{str(Path.home())}/Images/steg/base_img/{item['name']}"; 
        if (os.path.exists(file_name)):
           print("already downloaded, skiping...")
        else:
            print("downloading...")
            request = service.files().get_media(fileId=item['id'])
            file_data = io.BytesIO()
            downloader = MediaIoBaseDownload(file_data, request)
            done = False
            while done is False:
                status, done = downloader.next_chunk()
                print(f"Download {int(status.progress() * 100)}.")
            
            file = open(file_name,"wb")
            file.write(file_data.getbuffer())
            file.close()

  except HttpError as error:
    # TODO(developer) - Handle errors from drive API.
    print(f"An error occurred: {error}")


if __name__ == "__main__":
  main()