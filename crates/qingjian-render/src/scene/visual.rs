//! 场景图节点画什么：节点的盒子由布局算出，这里只描述在盒子里怎么画。

use super::icon::Icon;
use crate::color::Color;
use crate::text::TextStyle;

#[derive(Debug, Clone)]
pub(crate) enum Visual {
    /// 纯容器，自己不画。
    Group,

    /// 填满盒子的（圆角）矩形；`radius` 为 0 时画直角。
    Fill { color: Color, radius: f32 },

    /// 单行文字，盒子左上角是行框顶边；布局时按文字量宽高。
    Text { text: String, style: TextStyle },

    /// 图标：边长 `size`，在盒子里垂直居中、靠左。
    Icon { icon: Icon, size: f32, color: Color },
}
