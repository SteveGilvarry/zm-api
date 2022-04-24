import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Orientation } from '../monitors/monitors-orientation.enum';

@InputType()
export class EnumMonitors_OrientationFieldUpdateOperationsInput {

    @Field(() => Monitors_Orientation, {nullable:true})
    set?: keyof typeof Monitors_Orientation;
}
