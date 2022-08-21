import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesCreateInput } from './frames-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneFramesArgs {

    @Field(() => FramesCreateInput, {nullable:false})
    @Type(() => FramesCreateInput)
    data!: FramesCreateInput;
}
