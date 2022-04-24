import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { MonitorPresets_Type } from './monitor-presets-type.enum';

@InputType()
export class EnumMonitorPresets_TypeFieldUpdateOperationsInput {

    @Field(() => MonitorPresets_Type, {nullable:true})
    set?: keyof typeof MonitorPresets_Type;
}
