-- Sample tables and data for testing connected databases

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    age INTEGER,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    price NUMERIC(10, 2) NOT NULL,
    stock INTEGER DEFAULT 0,
    category VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    total_amount NUMERIC(10, 2),
    status VARCHAR(50) DEFAULT 'pending',
    ordered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (name, email, age, is_active) VALUES
    ('田中太郎', 'tanaka@example.com', 30, true),
    ('佐藤花子', 'sato@example.com', 25, true),
    ('鈴木一郎', 'suzuki@example.com', 35, false),
    ('高橋美咲', 'takahashi@example.com', 28, true),
    ('伊藤健太', 'ito@example.com', 42, true);

INSERT INTO products (name, description, price, stock, category) VALUES
    ('ノートパソコン', '高性能ビジネスノートPC', 89800.00, 15, 'Electronics'),
    ('ワイヤレスマウス', 'Bluetooth対応マウス', 3980.00, 50, 'Electronics'),
    ('プログラミング入門', 'Rustで学ぶプログラミング', 2980.00, 100, 'Books'),
    ('USBハブ', 'USB-C 4ポートハブ', 2480.00, 30, 'Electronics'),
    ('デスクライト', 'LED調光デスクライト', 5980.00, 20, 'Office');

INSERT INTO orders (user_id, product_id, quantity, total_amount, status) VALUES
    (1, 1, 1, 89800.00, 'completed'),
    (1, 2, 2, 7960.00, 'completed'),
    (2, 3, 1, 2980.00, 'pending'),
    (3, 4, 3, 7440.00, 'shipped'),
    (4, 5, 1, 5980.00, 'pending');
