use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc};


struct Philosopher {
    name: String,
    left: usize,// フォークの表現はベクトルのインデックスに対応するため、ここでは usize 型を使います
    right: usize,
}

impl Philosopher {
    fn new(name: &str, left: usize, right: usize) -> Philosopher {
        Philosopher {
            name: name.to_string(),
            left: left,
            right: right,
        }
    }
    // 新しい行が3つあります。
    // 新しい引数 table も追加しました。
    // Table が保持するフォークのリストにアクセスし、
    // フォークにアクセスするため self.left と self.right をインデクス値に用います。
    // そのインデクスから Mutex が得られたら、 lock() を呼び出します。
    // ミューテックスが別スレッドから並行アクセスされていた場合は、有効になるまでブロックされるでしょう。
    // またフォークを取上げる操作が一瞬で終わらないよう、
    // 最初のフォークを取上げてから2つ目のフォークを取上げるまでの間に thread::sleep を呼び出します。
    // lock() 呼び出しは失敗する可能性があり、その場合は、プログラムをクラッシュさせます。
    // この状況は、ミューテックスが 「poisoned」 状態、
    // つまりロック保持中のスレッドがパニックした場合にしか発生しません。
    // つまり今は起こりえないため、単に unwrap() を使っています。
    // もう一つの変わった点として: 結果を _left と _right と名づけました。
    // このアンダースコアはなにもの?
    // ええと、ロック内ではこれらの値を 使う 予定がありません。単にロックを獲得したいだけです。
    // そうなると、Rustは値が未使用だと警告してくるでしょう。
    // アンダースコアを使えば、Rustにこちらの意図を伝えることができ、 警告されなくなるのです。
    // ロックの解放はどうしましょう?はい、 _left と _right がスコープから抜けるとき、自動的に解放されます。
    fn eat(&self, table: &Table) {
        let _left = table.forks[self.left].lock().unwrap();
        thread::sleep(Duration::from_millis(150));
        let _right = table.forks[self.right].lock().unwrap();

        println!("{} is eating.", self.name);

        thread::sleep(Duration::from_millis(1000));

        println!("{} is done eating.", self.name);
    }
}


// この Table は Mutex のベクトルを保持します。
// ミューテックスは並行処理を制御するための機構です: その内容へ同時アクセスできるのは1スレッドに限定されます。
// これは正に今回のフォークに求められる性質です。
// 単に保持するだけで、実際に値を使うあても無いため、ミューテックスの中身は空タプル () とします。
struct Table {
    forks: Vec<Mutex<()>>,
}

fn main() {
    let table = Arc::new(Table { forks: vec![
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
    ]});
    // Philosopher のコンストラクタには left と right の値を渡す必要があります。
    // ここではもう1つ細かい話がありますが、 これは_非常に_重要な部分です。
    // 規則性という点では、最後以外は特に問題ありません。
    // ムッシュ・フーコー(Foucault)は 4, 0 を引数にとるべきですが、 代わりに、 0, 4 としています。
    // これはデッドロックを防ぐためのものです。実は: 哲学者の一人は左利きだったのです!
    // これは問題解決の一つのやり方ですが、私の見立てでは、最も単純な方法です。
    // 実引数の順番を変更すれば、デッドロックが生じるのを観測できるでしょう。
    let philosophers = vec![
        Philosopher::new("Judith Butler", 0, 1),
        Philosopher::new("Gilles Deleuze", 1, 2),
        Philosopher::new("Karl Marx", 2, 3),
        Philosopher::new("Emma Goldman", 3, 4),
        Philosopher::new("Michel Foucault", 0, 4),
    ];

    //
    let handles: Vec<_> = philosophers.into_iter().map(|p| {
        let table = table.clone();

        thread::spawn(move || {
            p.eat(&table);
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }
}