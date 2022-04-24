import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Orientation } from '../monitors/monitors-orientation.enum';
import { NestedEnumMonitors_OrientationFilter } from './nested-enum-monitors-orientation-filter.input';

@InputType()
export class EnumMonitors_OrientationFilter {

    @Field(() => Monitors_Orientation, {nullable:true})
    equals?: keyof typeof Monitors_Orientation;

    @Field(() => [Monitors_Orientation], {nullable:true})
    in?: Array<keyof typeof Monitors_Orientation>;

    @Field(() => [Monitors_Orientation], {nullable:true})
    notIn?: Array<keyof typeof Monitors_Orientation>;

    @Field(() => NestedEnumMonitors_OrientationFilter, {nullable:true})
    not?: NestedEnumMonitors_OrientationFilter;
}
