import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Frames_Type } from './frames-type.enum';

@InputType()
export class EnumFrames_TypeFieldUpdateOperationsInput {

    @Field(() => Frames_Type, {nullable:true})
    set?: keyof typeof Frames_Type;
}
