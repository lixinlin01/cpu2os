use candle_core::{Device, Result, Tensor, Module};
use candle_nn::{Optimizer, SGD, VarMap};

fn main() -> Result<()> {
    // 1. 選擇裝置
    let device = Device::Cpu;

    // 2. 生成數據
    let x_data = Tensor::new(&[[1f32], [2f32], [3f32], [4f32], [5f32]], &device)?;
    let y_data = Tensor::new(&[[3.1f32], [5.0f32], [6.9f32], [9.2f32], [10.8f32]], &device)?;

    // 3. 初始化參數空間與模型
    let varmap = VarMap::new();
    let vb = candle_nn::VarBuilder::from_varmap(&varmap, candle_core::DType::F32, &device);
    
    // 建立線性層 (1 In, 1 Out)
    let model = candle_nn::linear(1, 1, vb.pp("linear"))?;

    // 4. 設定優化器
    let mut sgd = SGD::new(varmap.all_vars(), 0.01)?;

    println!("開始訓練...");

    // 5. 訓練循環
    for epoch in 1..=500 {
        // 這裡現在可以正確呼叫 .forward() 了，因為導入了 Module Trait
        let prediction = model.forward(&x_data)?;
        
        let loss = prediction.sub(&y_data)?.sqr()?.mean_all()?;
        
        sgd.backward_step(&loss)?;

        if epoch % 100 == 0 {
            println!("Epoch: {}, Loss: {:.4}", epoch, loss.to_scalar::<f32>()?);
        }
    }

    // 6. 測試
    let test_x = Tensor::new(&[[10f32]], &device)?;
    let final_res = model.forward(&test_x)?;
    
    // 修正點：使用 flatten_all() 將 [1, 1] 轉為 [1]，再取第一個元素
    let val = final_res.flatten_all()?.get(0)?.to_scalar::<f32>()?;
    
    println!("---");
    println!("預測 x=10 的結果: {:.4}", val);
    println!("理論目標值: 21.0");

    Ok(())
}