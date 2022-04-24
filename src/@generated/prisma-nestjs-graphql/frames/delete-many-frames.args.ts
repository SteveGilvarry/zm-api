import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';

@ArgsType()
export class DeleteManyFramesArgs {

    @Field(() => FramesWhereInput, {nullable:true})
    where?: FramesWhereInput;
}
