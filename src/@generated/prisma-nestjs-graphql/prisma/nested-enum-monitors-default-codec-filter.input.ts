import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_DefaultCodec } from '../monitors/monitors-default-codec.enum';

@InputType()
export class NestedEnumMonitors_DefaultCodecFilter {

    @Field(() => Monitors_DefaultCodec, {nullable:true})
    equals?: keyof typeof Monitors_DefaultCodec;

    @Field(() => [Monitors_DefaultCodec], {nullable:true})
    in?: Array<keyof typeof Monitors_DefaultCodec>;

    @Field(() => [Monitors_DefaultCodec], {nullable:true})
    notIn?: Array<keyof typeof Monitors_DefaultCodec>;

    @Field(() => NestedEnumMonitors_DefaultCodecFilter, {nullable:true})
    not?: NestedEnumMonitors_DefaultCodecFilter;
}
