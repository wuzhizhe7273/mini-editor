use std::cmp;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    len:usize
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row=Self {
            string: String::from(slice),
            len:0
        };
        row.update_len();
        row
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        // 计算unicode字位
        let mut result=String::new();
        let graphemes = UnicodeSegmentation::graphemes(self.string.as_str(), true).skip(start).take(end-start).collect::<Vec<&str>>();
        for grapheme in graphemes{
            result.push_str(grapheme)
        }
        result    
    }
    pub fn len(&self) -> usize {
       self.len
    }
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len==0
    }

    pub fn update_len(&mut self){
        self.len = UnicodeSegmentation::graphemes(self.string.as_str(), true).collect::<Vec<&str>>().len();
    }
    // 插入字符
    pub fn insert(&mut self, at:usize, c:char){
        if at>self.len{
            self.string.push(c);
        }else {
            let mut result:String=UnicodeSegmentation::graphemes(self.string.as_str(), true).take(at).collect();
            let  remainder:String=UnicodeSegmentation::graphemes(self.string.as_str(), true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.string=result
        }
        self.update_len();
    }
    pub fn delete(&mut self, at:usize){
        if at>self.len(){
            return;
        }else {
            let mut result:String=UnicodeSegmentation::graphemes(self.string.as_str(), true).take(at).collect();
            let  remainder:String=UnicodeSegmentation::graphemes(self.string.as_str(), true).skip(at+1).collect();
            result.push_str(&remainder);
            self.string=result
        }
        self.update_len();
    }
    pub fn append(&mut self,new:&Self){
        self.string=format!("{}{}",self.string,new.string);
        self.update_len();
    }
    pub fn split(&mut self, at: usize) -> Self {
        let beginning: String = UnicodeSegmentation::graphemes(self.string.as_str(), true).take(at).collect();
        let remainder: String = UnicodeSegmentation::graphemes(self.string.as_str(), true).skip(at).collect();
        self.string = beginning;
        self.update_len();
        Self::from(&remainder[..])
    }
    pub fn as_bytes(&self)->&[u8]{
        self.string.as_bytes()
    }
}