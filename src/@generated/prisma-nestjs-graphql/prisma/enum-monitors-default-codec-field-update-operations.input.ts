import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_DefaultCodec } from '../monitors/monitors-default-codec.enum';

@InputType()
export class EnumMonitors_DefaultCodecFieldUpdateOperationsInput {

    @Field(() => Monitors_DefaultCodec, {nullable:true})
    set?: keyof typeof Monitors_DefaultCodec;
}
