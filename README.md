# Webcrawler

## High-level Approach

- The program starts in `main.rs`, where I collect the program arguments (username and password), and create the `HttpClient`, set of visited links, the total flags collected so far, and the work queue of links to visit (initialized with the starting URL).
- `HttpClient` is an object/struct I made to perform get and post requests, and maintain the socket so it can be kept alive using `"keep-alive"` across requests to speed things up.
- Before the webscraping can begin, we first need to log in, which is done in `login.rs`. This performs the get request for the login page, and then submits the credentials with the csrfmiddlewaretoken in the data field, along with the cookie recieved from the initial page.
- Then, webscraping begins, and the client makes requests, getting all the links and printing all flags, and dealing gracefully with any errors that arise.
- The program terminates once the fifth flag has been found and printed.

## Challenges

- There were a fair amount of small bugs like forgetting to update cookies or appending duplicate headers, as well as the learning curve of rust, since this was my first time using the language.
- I started with using `Connection: close`, but wanted to speed things up so decided to implement `keep-alive`, which presented challenges with only reading the correct amount from the socket, which meant i had to read out the headers first byte by byte, and then read the amount of data specified in the header.

## Testing

- For testing, the majority came from testing with the provided server for the complex tasks like performing HTTP requests
- I included some unit tests for sanity checks on some of the utility functions like in `parse.rs` to make sure they are in fact working.
