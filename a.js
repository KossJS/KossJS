let a = "Hello, From KossJS!";
console.log(a);
console.log(a.length);
for(let i = 0; i<=a.length; i++) {
  console.log(a.substring(0, i));
}

(async () => {
  let reps = await fetch('https://www.bilibili.com/')
  if (reps.ok) {
    let text = await reps.text()
    console.log(text)
  }
})()
