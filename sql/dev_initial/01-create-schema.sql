-- Users
CREATE TABLE users
(
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START 1) PRIMARY KEY,
    username varchar(128) NOT NULL UNIQUE,
    -- Auth
    pwd varchar(256),
    pwd_salt uuid NOT NULL DEFAULT gen_random_uuid(),
    token_salt uuid NOT NULL DEFAULT gen_random_uuid()
);

-- Task
CREATE TABLE task
(
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1) PRIMARY KEY,
    title varchar(256) NOT NULL
);

-- Chat Rooms
CREATE TABLE rooms
(
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1) PRIMARY KEY,
    title varchar(256) NOT NULL

);

-- Chat Messages
CREATE TABLE messages
(
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1) PRIMARY KEY,
    message_text TEXT,
    message_datetime TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    message_room_id BIGINT NOT NULL, --Indicates which room the message belongs to
    message_user_name varchar(128) NOT NULL, --Indicates the user who sent the message
    
    FOREIGN KEY (message_room_id) REFERENCES rooms(id) ON DELETE CASCADE,
    FOREIGN KEY (message_user_name) REFERENCES users(username) ON DELETE CASCADE
);

-- Friends
CREATE TABLE friends
(
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1) PRIMARY KEY,
    user1_name varchar(128) NOT NULL,
    user2_name varchar(128) NOT NULL,
    status varchar(50) DEFAULT 'PENDING', -- Status could be PENDING, ACCEPTED, REJECTED, etc.
    
    FOREIGN KEY (user1_name) REFERENCES users(username) ON DELETE CASCADE,
    FOREIGN KEY (user2_name) REFERENCES users(username) ON DELETE CASCADE
);

-- Private Messages
CREATE TABLE private_messages
(
    id BIGINT GENERATED BY DEFAULT AS IDENTITY (START WITH 1) PRIMARY KEY,
    sender_name varchar(128) NOT NULL,
    receiver_name varchar(128) NOT NULL,
    message_text TEXT,
    message_datetime TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (sender_name) REFERENCES users(username) ON DELETE CASCADE,
    FOREIGN KEY (receiver_name) REFERENCES users(username) ON DELETE CASCADE
);
