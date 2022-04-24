import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereUniqueInput } from './frames-where-unique.input';

@ArgsType()
export class DeleteOneFramesArgs {

    @Field(() => FramesWhereUniqueInput, {nullable:false})
    where!: FramesWhereUniqueInput;
}
