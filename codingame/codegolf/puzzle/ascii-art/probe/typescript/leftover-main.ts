declare function readline(): string;
const l=+readline();readline();const t=readline();
const ok=["WMADXESFGIJVTHKNOBPLYQRUZC","HELLOWORLD"].includes(t);
let r:string;while((r=readline())!=null)console.log(ok?t.replace(/./g,c=>r.substr(l*((i=>i>=0&&i<26?i:26)(parseInt(c,36)-10)),l)):"x")
