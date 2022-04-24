import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesCreateInput } from './frames-create.input';

@ArgsType()
export class CreateOneFramesArgs {

    @Field(() => FramesCreateInput, {nullable:false})
    data!: FramesCreateInput;
}
