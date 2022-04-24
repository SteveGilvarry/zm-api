import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Orientation } from '../monitors/monitors-orientation.enum';

@InputType()
export class NestedEnumMonitors_OrientationFilter {

    @Field(() => Monitors_Orientation, {nullable:true})
    equals?: keyof typeof Monitors_Orientation;

    @Field(() => [Monitors_Orientation], {nullable:true})
    in?: Array<keyof typeof Monitors_Orientation>;

    @Field(() => [Monitors_Orientation], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Orientation>;

    @Field(() => NestedEnumMonitors_OrientationFilter, {nullable:true})
    not?: NestedEnumMonitors_OrientationFilter;
}
